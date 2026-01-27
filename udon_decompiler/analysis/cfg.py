from dataclasses import dataclass, field
from typing import Dict, List, Optional, Set

import networkx as nx
from pydot import Dot

from udon_decompiler.analysis.basic_block import BasicBlock, BasicBlockIdentifier
from udon_decompiler.analysis.stack_simulator import (
    BlockStackSimulator,
    HeapSimulator,
    StackFrame,
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
    graph: nx.DiGraph = field(default_factory=nx.DiGraph)

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

    def get_reachable_blocks(self) -> Set[BasicBlock]:
        return set(nx.descendants(self.graph, self.entry_block)) | {self.entry_block}

    def get_dominators(self) -> Dict[BasicBlock, Set[BasicBlock]]:
        idom = nx.immediate_dominators(self.graph, self.entry_block)

        doms = {}
        for node in idom:
            doms[node] = set()
            current = node
            while current in idom and idom[current] != current:
                current = idom[current]
                doms[node].add(current)

        return doms

    def get_post_dominators(self) -> Dict[BasicBlock, Set[BasicBlock]]:
        reversed_graph = self.graph.reverse()
        exit_nodes = [
            node for node in self.graph.nodes() if self.graph.out_degree(node) == 0
        ]

        if not exit_nodes:
            return {}

        virtual_exit = BasicBlock(start_address=-1, end_address=-1)
        reversed_graph.add_node(virtual_exit)
        for exit_node in exit_nodes:
            reversed_graph.add_edge(virtual_exit, exit_node)

        idom = nx.immediate_dominators(reversed_graph, virtual_exit)

        post_doms = {}
        for node in idom:
            post_doms[node] = set()
            current = node
            while current in idom and idom[current] != current:
                current = idom[current]
                if current != virtual_exit:
                    post_doms[node].add(current)

        return post_doms

    def get_immediate_post_dominator(self, block: BasicBlock) -> Optional[BasicBlock]:
        reversed_graph = self.graph.reverse()
        exit_nodes = [
            node for node in self.graph.nodes() if self.graph.out_degree(node) == 0
        ]

        if not exit_nodes:
            return None

        virtual_exit = BasicBlock(start_address=-1, end_address=-1)
        reversed_graph.add_node(virtual_exit)
        for exit_node in exit_nodes:
            reversed_graph.add_edge(virtual_exit, exit_node)

        idom = nx.immediate_dominators(reversed_graph, virtual_exit)
        candidate = idom.get(block)
        if candidate == virtual_exit:
            return None
        return candidate

    def get_back_edges(self) -> List[tuple[BasicBlock, BasicBlock]]:
        try:
            back_edges = []
            dominators = self.get_dominators()

            for src, dst in self.graph.edges():
                if dst in dominators.get(src, set()):
                    back_edges.append((src, dst))

            return back_edges
        except Exception:
            return []

    def get_loops(self) -> List[Set[BasicBlock]]:
        loops = []
        back_edges = self.get_back_edges()

        for src, dst in back_edges:
            loop_nodes = {dst}
            stack = [src]
            visited = {dst}

            while stack:
                node = stack.pop()
                if node in visited:
                    continue

                visited.add(node)
                loop_nodes.add(node)

                for pred in self.get_predecessors(node):
                    if pred not in visited:
                        stack.append(pred)

            loops.append(loop_nodes)

        return loops

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

    def build(self) -> Dict[str, ControlFlowGraph]:
        logger.debug("Building control flow graphs...")

        self._identify_entry_points()
        self._identify_hidden_entry_points()
        self._build_edges()

        self._cfgs = self._build_function_cfgs()

        logger.info(f"Built {len(self._cfgs)} control flow graphs")
        logger.info(f"Program entry points identified: {self.program.entry_points}")

        return self._cfgs

    def _identify_entry_points(self):
        res = set(self.program.entry_points)
        for inst in self.instructions:
            if inst.opcode != OpCode.PUSH:
                continue
            if inst.operand is None:
                raise Exception("Invalid PUSH instruction! An operand expected!")

            sym = self.program.get_symbol_by_address(inst.operand)
            if sym.name != SymbolInfo.HALT_JUMP_ADDR_SYMBOL_NAME:
                continue
            val = self.program.get_initial_heap_value(inst.operand)
            if val is None:
                raise Exception("Invalid symbol! Initial value not found!")
            if val.value.value != Instruction.HALT_JUMP_ADDR:
                continue

            # this is a halt jump target! the next inst is a function entry!
            res.add(
                EntryPointInfo(
                    name=None,
                    address=inst.address,
                    call_jump_target=inst.next_address,
                )
            )

        self.program.entry_points = list(res)

    def _identify_hidden_entry_points(self) -> None:
        updated = True
        while updated:
            updated = False
            entry_addresses = []
            for e in self.program.entry_points:
                if e.address is None:
                    entry_addresses.append(e.call_jump_target)
                else:
                    entry_addresses.append(e.address)

            self.identifier = BasicBlockIdentifier(
                self.instructions, entry_addresses, self.program
            )
            self._all_blocks = self.identifier.identify()
            logger.debug(f"Basic Blocks: {self._all_blocks}")

            self._address_to_block = {}
            for block in self._all_blocks:
                self._address_to_block[block.start_address] = block

            new_entry_points = self._find_call_targets(self._all_blocks)
            updated = self._merge_entry_points(new_entry_points)

    def _find_call_targets(self, blocks: List[BasicBlock]) -> List[EntryPointInfo]:
        """
        Find call jumps from basic blocks whose target is not listed in known
        entry points, and thus find hidden entry points.
        """
        simulator = BlockStackSimulator(self.program)
        new_entries: Dict[int, EntryPointInfo] = {}

        for block in blocks:
            state = StackFrame()
            heap = HeapSimulator(self.program)

            if block.last_instruction.opcode != OpCode.JUMP:
                continue

            for inst in block.instructions:
                if inst.opcode != OpCode.JUMP:
                    simulator.step(inst, state, heap)
                    continue

                # when calling a function, there should be an address in the stack
                # that points to the next instruction to the call jump
                top = state.peek(0)
                if top is None or top.literal_value != inst.next_address:
                    # this is not a call jump
                    continue

                target = inst.get_jump_target()
                new_entries[target] = EntryPointInfo(
                    name=None,
                    address=target,
                    call_jump_target=target,
                )

        return list(new_entries.values())

    def _merge_entry_points(self, new_entries: List[EntryPointInfo]) -> bool:
        jump_targets = [ep.call_jump_target for ep in self.program.entry_points]
        changed = False
        for entry in new_entries:
            if entry.call_jump_target in jump_targets:
                continue
            self.program.entry_points.append(entry)
            jump_targets.append(entry.call_jump_target)
            changed = True
        return changed

    def _build_edges(self) -> None:
        for block in self._all_blocks:
            last_inst = block.last_instruction

            if last_inst.address in self.identifier.return_jumps:
                # return jumps are ignored, since we only care about jumps
                # inside a function
                continue

            if last_inst.opcode == OpCode.JUMP:
                target = last_inst.get_jump_target()
                is_call_jump = any(
                    ep.call_jump_target == target for ep in self.program.entry_points
                )
                if is_call_jump:
                    returning_block = self._get_block_starting_at(
                        last_inst.next_address
                    )
                    block.add_successor(returning_block)
                    returning_block.add_predecessor(block)
                else:
                    target_block = self._get_block_starting_at(target)
                    block.add_successor(target_block)
                    target_block.add_predecessor(block)

            elif last_inst.opcode == OpCode.JUMP_IF_FALSE:
                target = last_inst.get_jump_target()

                # "FALSE" branch
                target_block = self._get_block_starting_at(target)
                block.add_successor(target_block)
                target_block.add_predecessor(block)

                # "TRUE" branch
                next_addr = last_inst.next_address
                next_block = self._get_block_starting_at(next_addr)
                block.add_successor(next_block)
                next_block.add_predecessor(block)

            elif last_inst.opcode == OpCode.JUMP_INDIRECT:
                if last_inst.operand is not None:
                    switch_info = self.identifier.switch_cases_indir_jumps.get(
                        last_inst.address
                    )
                    if switch_info:
                        block.switch_info = switch_info
                        for target in switch_info.targets:
                            next_block = self._get_block_starting_at(target)
                            block.add_successor(next_block)
                            next_block.add_predecessor(block)

            else:
                next_addr = last_inst.next_address
                next_block = self._get_block_starting_at(next_addr)
                block.add_successor(next_block)
                next_block.add_predecessor(block)

        logger.debug("CFG edges built successfully")

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

        # find a --copy-> __*___*_name__ret
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
