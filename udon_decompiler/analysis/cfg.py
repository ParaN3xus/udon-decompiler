from dataclasses import dataclass, field
from typing import Dict, List, Optional, Set

import networkx as nx
from pydot import Dot

from udon_decompiler.analysis.basic_block import BasicBlock, BasicBlockIdentifier
from udon_decompiler.models.instruction import Instruction, OpCode
from udon_decompiler.models.program import EntryPointInfo, SymbolInfo, UdonProgramData
from udon_decompiler.utils.logger import logger
from udon_decompiler.utils.utils import sliding_window


@dataclass
class ControlFlowGraph:
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
    def __init__(self, program: UdonProgramData, instructions: List[Instruction]):
        self.program = program
        self.instructions = instructions
        self._cfgs: Dict[str, ControlFlowGraph] = {}
        self._all_blocks: List[BasicBlock] = []
        self._address_to_block: Dict[int, BasicBlock] = {}

    def build(self) -> Dict[str, ControlFlowGraph]:
        logger.info("Building control flow graphs...")

        self._identify_entry_points()
        entry_addresses = [e.address for e in self.program.entry_points]
        self.identifier = BasicBlockIdentifier(
            self.instructions, entry_addresses, self.program
        )
        self._all_blocks = self.identifier.identify()
        logger.debug(f"Basic Blocks: {self._all_blocks}")

        for block in self._all_blocks:
            self._address_to_block[block.start_address] = block

        self._build_edges()

        self._cfgs = self._build_function_cfgs()

        logger.info(f"Built {len(self._cfgs)} control flow graphs")

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

            # this is a halt jump! the next inst is a function entry!
            res.add(EntryPointInfo(name=None, address=inst.address))

        self.program.entry_points = list(res)

    def _build_edges(self) -> None:
        logger.info("Building CFG edges...")

        for block in self._all_blocks:
            last_inst = block.last_instruction

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
                if target is not None:
                    target_block = self._get_block_starting_at(target)
                    block.add_successor(target_block)
                    target_block.add_predecessor(block)

                next_addr = last_inst.next_address
                next_block = self._get_block_starting_at(next_addr)
                block.add_successor(next_block)
                next_block.add_predecessor(block)

            elif last_inst.opcode == OpCode.JUMP_INDIRECT:
                if last_inst.operand is not None:
                    if last_inst.address in self.identifier.return_indir_jumps:
                        # return jumps are ignored, since we only care about jumps
                        # inside a function
                        continue
                    if last_inst.address in self.identifier.switch_cases_indir_jumps:
                        targets = self.identifier.switch_cases_indir_jumps[
                            last_inst.address
                        ]
                        for target in targets:
                            next_block = self._get_block_starting_at(target)
                            block.add_successor(next_block)
                            next_block.add_predecessor(block)

            else:
                next_addr = last_inst.next_address
                next_block = self._get_block_starting_at(next_addr)
                block.add_successor(next_block)
                next_block.add_predecessor(block)

        logger.info("CFG edges built successfully")

    def _get_block_starting_at(self, address: int) -> BasicBlock:
        return self._address_to_block[address]

    def _build_function_cfgs(self) -> Dict[str, ControlFlowGraph]:
        cfgs = {}

        for entry_point in self.program.entry_points:
            function_name = entry_point.name
            entry_block = self._address_to_block.get(entry_point.address)

            if entry_block is None:
                logger.warning(
                    f"Cannot find entry block for function {function_name} "
                    f"at address 0x{entry_point.address:08x}"
                )
                continue

            function_blocks = self._find_function_blocks(entry_block)
            if function_name is None:
                self._identify_function_name(entry_point, function_blocks)
                function_name = entry_point.name

            cfg = ControlFlowGraph(entry_block=entry_block, function_name=function_name)

            logger.debug(f"Basic blocks of function {function_name}: {function_blocks}")

            for block in function_blocks:
                block.function_name = function_name
                cfg.graph.add_node(block)

            for block in function_blocks:
                for successor in block.successors:
                    if successor in function_blocks:
                        cfg.graph.add_edge(block, successor)

            cfgs[function_name] = cfg
            logger.info(
                f"Built CFG for function {function_name}: {len(function_blocks)} blocks"
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
