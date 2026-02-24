from collections import deque
from dataclasses import dataclass, field
from typing import Deque, Dict, List, Optional, Set, Tuple

import networkx as nx
from pydot import Dot

from udon_decompiler.analysis.basic_block import (
    BasicBlock,
    BasicBlockIdentifier,
    BasicBlockType,
)
from udon_decompiler.analysis.stack_simulator import (
    BlockStackSimulator,
    HeapSimulator,
    StackFrame,
    StackValue,
)
from udon_decompiler.models.instruction import Instruction, OpCode
from udon_decompiler.models.program import EntryPointInfo, SymbolInfo, UdonProgramData
from udon_decompiler.utils.logger import logger
from udon_decompiler.utils.utils import sliding_window


@dataclass
class ControlFlowGraph:
    is_function_public: bool
    entry_block: BasicBlock
    function_name: Optional[str] = None
    graph: "nx.DiGraph[BasicBlock]" = field(default_factory=nx.DiGraph)

    def __repr__(self) -> str:
        return (
            f"CFG(function={self.function_name}, "
            f"entry=0x{self.entry_block.start_address:08x}, "
            f"blocks={self.graph.number_of_nodes()})"
        )

    def get_block_at(self, address: int) -> Optional[BasicBlock]:
        for block in self.graph.nodes():
            block: BasicBlock
            if block.contains_address(address):
                return block
        return None

    def get_successors(self, block: BasicBlock) -> List[BasicBlock]:
        return list(self.graph.successors(block))

    def get_predecessors(self, block: BasicBlock) -> List[BasicBlock]:
        return list(self.graph.predecessors(block))

    def to_dot(self) -> Dot:
        return nx.nx_pydot.to_pydot(self.graph)


class CFGBuilder:
    def __init__(
        self,
        program: UdonProgramData,
        instructions: List[Instruction],
    ):
        self.program = program
        self.instructions = instructions
        self._cfgs: Dict[str, ControlFlowGraph] = {}
        self._all_blocks: List[BasicBlock] = []
        self._address_to_block: Dict[int, BasicBlock] = {}
        self._address_to_instruction = {inst.address: inst for inst in instructions}
        self._prev_instruction_by_address: Dict[int, Optional[Instruction]] = {}
        prev: Optional[Instruction] = None
        for inst in instructions:
            self._prev_instruction_by_address[inst.address] = prev
            prev = inst
        self._entry_targets: Set[int] = set()
        self._block_simulator = BlockStackSimulator(program)

    def build(self) -> Dict[str, ControlFlowGraph]:
        logger.debug("Building control flow graphs...")

        self._build_initial_blocks()
        self._build_edges()

        self._cfgs = self._build_function_cfgs()

        logger.info(f"Built {len(self._cfgs)} control flow graphs")
        logger.info(f"Program entry points identified: {self.program.entry_points}")

        return self._cfgs

    def _build_initial_blocks(self) -> None:
        entry_addresses = []
        for ep in self.program.entry_points:
            entry_addresses.append(ep.call_jump_target)
            if ep.call_jump_target != ep.address:
                entry_addresses.append(ep.address)

        self.identifier = BasicBlockIdentifier(
            self.instructions, entry_addresses, self.program
        )
        self._all_blocks = self.identifier.identify()
        logger.debug(f"Basic Blocks: {self._all_blocks}")

        self._address_to_block = {}
        for block in self._all_blocks:
            self._address_to_block[block.start_address] = block

    def _build_edges(self) -> None:
        for block in self._all_blocks:
            block.predecessors = []
            block.successors = []

        self._entry_targets = {ep.call_jump_target for ep in self.program.entry_points}

        pending: Deque[Tuple[BasicBlock, StackFrame, HeapSimulator]] = deque()
        visited_blocks: Set[BasicBlock] = set()

        for entry in self.program.entry_points:
            entry_block = self._address_to_block.get(entry.address)
            if entry_block is None:
                continue
            init = self._initial_state_for_entry(entry)
            pending.append((entry_block, init, HeapSimulator(self.program)))

        while pending:
            block, entry_state, entry_heap = pending.popleft()
            if block in visited_blocks:
                continue
            visited_blocks.add(block)
            self._process_block_with_state(
                block,
                entry_state,
                entry_heap,
                pending,
            )

        self.program.entry_points.sort()

        logger.debug("CFG edges built successfully")

    def _process_block_with_state(
        self,
        block: BasicBlock,
        entry_state: StackFrame,
        entry_heap: HeapSimulator,
        pending: Deque[Tuple[BasicBlock, StackFrame, HeapSimulator]],
    ) -> None:
        state = entry_state.copy()
        heap = entry_heap.copy()

        for instruction in block.instructions:
            opcode = instruction.opcode

            if opcode == OpCode.JUMP:
                target = instruction.get_jump_target()
                top = state.peek(0)

                seems_like_call = (
                    target in self._entry_targets
                    or self._looks_like_function_header(target)
                )
                is_returning_call = self._matches_literal(
                    top, instruction.next_address, heap=heap
                )

                if is_returning_call:
                    self._register_entry_target(target, pending)
                    _ = state.pop()
                    self._add_fallthrough_edge(
                        block,
                        instruction.next_address,
                        state,
                        heap,
                        pending,
                    )
                    block.block_type = BasicBlockType.NORMAL
                    return

                # seems like call, but not a returning call -> terminal call
                if seems_like_call:
                    self._register_entry_target(target, pending)
                    block.block_type = BasicBlockType.RETURN
                    return

                # common jump
                self._add_jump_edge(block, target, state, heap, pending)
                block.block_type = BasicBlockType.JUMP
                return

            if opcode == OpCode.JUMP_IF_FALSE:
                state.pop()
                self._add_jump_edge(
                    block,
                    instruction.get_jump_target(),
                    state,
                    heap,
                    pending,
                )
                self._add_fallthrough_edge(
                    block,
                    instruction.next_address,
                    state,
                    heap,
                    pending,
                )
                block.block_type = BasicBlockType.CONDITIONAL
                return

            if opcode == OpCode.JUMP_INDIRECT:
                if instruction.operand is None:
                    raise Exception(
                        "Invalid JUMP_INDIRECT instruction: missing operand!"
                    )

                operand_sym = self.program.get_symbol_by_address(instruction.operand)
                if operand_sym.name == SymbolInfo.RETURN_JUMP_ADDR_SYMBOL_NAME:
                    block.block_type = BasicBlockType.RETURN
                    return

                block.block_type = BasicBlockType.JUMP
                switch_info = self.identifier.switch_cases_indir_jumps.get(
                    instruction.address
                )
                if switch_info is None:
                    logger.warning(
                        "Unrecognized JUMP_INDIRECT encountered at %s! Ignoring..."
                    )
                    return
                block.switch_info = switch_info
                for target in switch_info.targets:
                    self._add_jump_edge(block, target, state, heap, pending)
                return

            state = self._block_simulator.step(instruction, state, heap)

        self._add_fallthrough_edge(
            block,
            block.last_instruction.next_address,
            state,
            heap,
            pending,
        )
        block.block_type = BasicBlockType.NORMAL

    def _initial_state_for_entry(self, entry: EntryPointInfo) -> StackFrame:
        if entry.address != entry.call_jump_target:
            return StackFrame()
        return StackFrame(
            [
                StackValue(
                    value=-1,
                    type_hint=None,
                    source_instruction=None,
                    literal_value=Instruction.HALT_JUMP_ADDR,
                )
            ]
        )

    def _add_jump_edge(
        self,
        source: BasicBlock,
        target_addr: int,
        state: StackFrame,
        heap: HeapSimulator,
        pending: Deque[Tuple[BasicBlock, StackFrame, HeapSimulator]],
    ) -> None:
        target = self._address_to_block.get(target_addr)
        if target is None:
            source.block_type = BasicBlockType.RETURN
            return
        self._connect_blocks(source, target)
        pending.append((target, state.copy(), heap.copy()))

    def _add_fallthrough_edge(
        self,
        source: BasicBlock,
        next_addr: int,
        state: StackFrame,
        heap: HeapSimulator,
        pending: Deque[Tuple[BasicBlock, StackFrame, HeapSimulator]],
    ) -> None:
        if next_addr >= self.program.byte_code_length:
            return
        target = self._address_to_block.get(next_addr)
        if target is None:
            raise Exception("Invalid fallthrough! Basic block excepted!")
        self._connect_blocks(source, target)
        pending.append((target, state.copy(), heap.copy()))

    @staticmethod
    def _connect_blocks(source: BasicBlock, target: BasicBlock) -> None:
        if target not in source.successors:
            source.add_successor(target)
        if source not in target.predecessors:
            target.add_predecessor(source)

    def _register_entry_target(
        self,
        target: int,
        pending: Deque[Tuple[BasicBlock, StackFrame, HeapSimulator]],
    ) -> None:
        if target in self._entry_targets:
            return

        self._entry_targets.add(target)
        self.program.entry_points.append(
            EntryPointInfo(name=None, address=target, call_jump_target=target)
        )

        target_block = self._address_to_block.get(target)
        if target_block is None:
            raise Exception("Invalid entry target! Basic block excepted!")
        pending.append(
            (
                target_block,
                self._initial_state_for_hidden_entry(),
                HeapSimulator(self.program),
            )
        )

    @staticmethod
    def _initial_state_for_hidden_entry() -> StackFrame:
        return StackFrame(
            [
                StackValue(
                    value=-1,
                    type_hint=None,
                    source_instruction=None,
                    literal_value=Instruction.HALT_JUMP_ADDR,
                )
            ]
        )

    def _looks_like_function_header(self, target: int) -> bool:
        header_addr = target - OpCode.PUSH.size
        return self._is_header_push_address(header_addr)

    def _matches_literal(
        self,
        stack_value: Optional[StackValue],
        expected: int,
        heap: HeapSimulator,
    ) -> bool:
        if stack_value is None:
            return False
        if stack_value.literal_value == expected:
            return True
        if heap.read_literal_int(stack_value.value) == expected:
            return True

        initial = self.program.get_initial_heap_value(stack_value.value)
        if initial is None or not initial.value.is_serializable:
            return False
        literal = initial.value.value
        return isinstance(literal, int) and literal == expected

    def _is_header_push_address(self, address: int) -> bool:
        inst = self._address_to_instruction.get(address)
        if inst is None or inst.opcode != OpCode.PUSH or inst.operand is None:
            return False
        try:
            symbol = self.program.get_symbol_by_address(inst.operand)
        except Exception:
            return False
        if symbol.name != SymbolInfo.HALT_JUMP_ADDR_SYMBOL_NAME:
            return False
        value = self.program.get_initial_heap_value(inst.operand)
        if value is None or value.value.value != Instruction.HALT_JUMP_ADDR:
            return False

        prev = self._prev_instruction_by_address.get(address)
        if prev is None:
            return True
        if prev.opcode != OpCode.JUMP_INDIRECT or prev.operand is None:
            return False
        try:
            prev_sym = self.program.get_symbol_by_address(prev.operand)
        except Exception:
            return False
        return prev_sym.name == SymbolInfo.RETURN_JUMP_ADDR_SYMBOL_NAME

    def _get_block_starting_at(self, address: int) -> BasicBlock:
        return self._address_to_block[address]

    def _build_function_cfgs(self) -> Dict[str, ControlFlowGraph]:
        cfgs = {}

        for entry_point in self.program.entry_points:
            entry_block = self._address_to_block.get(entry_point.address)

            if entry_block is None:
                logger.warning(
                    f"Cannot find entry block for function "
                    f"at address 0x{entry_point.address:08x}"
                )
                continue

            function_blocks = self._find_function_blocks(entry_block)
            is_function_public = True
            if entry_point.name is None:
                is_function_public = False
                self._identify_function_name(entry_point, function_blocks)

            cfg = ControlFlowGraph(
                entry_block=entry_block,
                function_name=entry_point.name,
                is_function_public=is_function_public,
            )

            logger.debug(
                f"Basic blocks of function {entry_point.name}: {function_blocks}"
            )

            for block in function_blocks:
                block.function_name = entry_point.name
                cfg.graph.add_node(block)

            for block in function_blocks:
                for successor in block.successors:
                    if successor in function_blocks:
                        cfg.graph.add_edge(block, successor)

            cfgs[entry_point.name] = cfg
            logger.debug(
                f"Built CFG for function {entry_point.name}: "
                f"{len(function_blocks)} blocks"
            )

        return cfgs

    def _find_function_blocks(self, entry_block: BasicBlock) -> List[BasicBlock]:
        """
        Return all blocks of the function that begin with the given entry_block.
        Order guaranteed.
        """
        visited = set()
        stack = [entry_block]

        while stack:
            block = stack.pop()
            if block in visited:
                continue

            visited.add(block)

            for successor in block.successors:
                stack.append(successor)

        res = list(visited)
        res.sort()
        return res

    def _identify_function_name(
        self, entry_point: EntryPointInfo, function_blocks: List[BasicBlock]
    ):
        instructions = [
            inst for block in function_blocks for inst in block.instructions
        ]

        candidate_names = []

        # todo: there can be other types of assigning ret
        # like
        # __0___0_fibonacci__ret <- __intnl_SystemInt32_2 + ret
        # PUSH, __intnl_SystemInt32_2
        # PUSH, __0___0_fibonacci__ret
        # PUSH, __0___0_fibonacci__ret
        # EXTERN, "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"

        # find a --copy-> __*_{__* | get}_name__ret
        for _, inst2, inst3 in sliding_window(instructions, 3):
            if inst3.opcode != OpCode.COPY:
                continue
            if inst2.opcode != OpCode.PUSH:
                logger.warning(
                    "Detected orphan COPY without a near PUSH. "
                    "The program may be broken."
                )
                continue
            if inst2.operand is None:
                raise Exception("Invalid PUSH instruction! An operand expected!")
            sym = self.program.get_symbol_by_address(inst2.operand)
            if sym is None:
                continue
            method_name = SymbolInfo.try_parse_function_return(sym.name)
            if method_name is not None:
                candidate_names.append(method_name)

        if candidate_names:
            distinct_names = set(candidate_names)
            if len(distinct_names) > 1:
                logger.warning(
                    f"Conflicting function names detected for entry point at "
                    f"{entry_point.address}: {distinct_names}. "
                    f"Using the first found name: '{candidate_names[0]}'."
                )

            entry_point.name = candidate_names[0]
            return

        entry_point.name = f"function_{self.program._generated_func_id}"
        self.program._generated_func_id += 1

    @property
    def cfgs(self) -> Dict[str, ControlFlowGraph]:
        return self._cfgs
