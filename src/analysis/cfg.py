from dataclasses import dataclass, field
from typing import List, Dict, Set, Optional
import networkx as nx
from pydot import Dot

from ..models.instruction import Instruction, OpCode
from ..models.program import UdonProgramData
from .basic_block import BasicBlock, BasicBlockIdentifier, BasicBlockType
from ..utils.logger import logger


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
        return nx.immediate_dominators(self.graph, self.entry_block)

    def get_post_dominators(self) -> Dict[BasicBlock, Set[BasicBlock]]:
        reversed_graph = self.graph.reverse()
        exit_nodes = [node for node in self.graph.nodes(
        ) if self.graph.out_degree(node) == 0]

        if not exit_nodes:
            return {}

        virtual_exit = BasicBlock(start_address=-1, end_address=-1)
        reversed_graph.add_node(virtual_exit)
        for exit_node in exit_nodes:
            reversed_graph.add_edge(virtual_exit, exit_node)

        return nx.immediate_dominators(reversed_graph, virtual_exit)

    def get_back_edges(self) -> List[tuple[BasicBlock, BasicBlock]]:
        try:
            back_edges = []
            dominators = self.get_dominators()

            for src, dst in self.graph.edges():
                if dst in dominators.get(src, set()):
                    back_edges.append((src, dst))

            return back_edges
        except:
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
        instructions: List[Instruction]
    ):
        self.program = program
        self.instructions = instructions
        self._cfgs: Dict[str, ControlFlowGraph] = {}
        self._all_blocks: List[BasicBlock] = []
        self._address_to_block: Dict[int, BasicBlock] = {}

    def build(self) -> Dict[str, ControlFlowGraph]:
        logger.info("Building control flow graphs...")

        entry_addresses = [ep.address for ep in self.program.entry_points]
        identifier = BasicBlockIdentifier(
            self.instructions, entry_addresses, self.program.heap_initial_values)
        self._all_blocks = identifier.identify()

        for block in self._all_blocks:
            self._address_to_block[block.start_address] = block

        self._build_edges()

        self._cfgs = self._build_function_cfgs()

        logger.info(f"Built {len(self._cfgs)} control flow graphs")

        return self._cfgs

    def _build_edges(self) -> None:
        logger.info("Building CFG edges...")

        for block in self._all_blocks:
            last_inst = block.last_instruction
            if not last_inst:
                continue

            if last_inst.opcode == OpCode.JUMP:
                target = last_inst.get_jump_target()
                if target is not None:
                    target_block = self._get_block_starting_at(target)
                    if target_block:
                        block.add_successor(target_block)
                        target_block.add_predecessor(block)

            elif last_inst.opcode == OpCode.JUMP_IF_FALSE:
                target = last_inst.get_jump_target()
                if target is not None:
                    target_block = self._get_block_starting_at(target)
                    if target_block:
                        block.add_successor(target_block)
                        target_block.add_predecessor(block)

                next_addr = last_inst.next_address
                next_block = self._get_block_starting_at(next_addr)
                if next_block:
                    block.add_successor(next_block)
                    next_block.add_predecessor(block)

            elif last_inst.opcode == OpCode.JUMP_INDIRECT:
                if last_inst.operand is not None:
                    heap_entry = self.program.get_initial_heap_value(
                        last_inst.operand)
                    if heap_entry and heap_entry.value.is_serializable:
                        target = heap_entry.value.value
                        if isinstance(target, int):
                            target_block = self._get_block_starting_at(target)
                            if target_block:
                                block.add_successor(target_block)
                                target_block.add_predecessor(block)
                                logger.debug(
                                    f"Resolved indirect jump at 0x{last_inst.address:08x}"
                                    f"to 0x{target:08x}"
                                )
                            else:
                                logger.warning(
                                    f"Unable to resolve target basic block of the indirect jump at 0x{last_inst.address:08x}")
                        else:
                            logger.warning(
                                f"Indirect jump at 0x{last_inst.address:08x}, "
                                f"heap value is not an integer"
                            )

            else:
                next_addr = last_inst.next_address
                next_block = self._get_block_starting_at(next_addr)
                if next_block:
                    block.add_successor(next_block)
                    next_block.add_predecessor(block)

        logger.info("CFG edges built successfully")

    def _get_block_starting_at(self, address: int) -> Optional[BasicBlock]:
        return self._address_to_block.get(address)

    def _build_function_cfgs(self) -> Dict[str, ControlFlowGraph]:
        cfgs = {}

        for entry_point in self.program.entry_points:
            function_name = entry_point.name
            entry_block = self._address_to_block.get(entry_point.address)

            if not entry_block:
                logger.warning(
                    f"Cannot find entry block for function {function_name} "
                    f"at address 0x{entry_point.address:08x}"
                )
                continue

            cfg = ControlFlowGraph(
                entry_block=entry_block,
                function_name=function_name
            )

            function_blocks = self._find_function_blocks(entry_block)

            for block in function_blocks:
                block.function_name = function_name
                cfg.graph.add_node(block)

            for block in function_blocks:
                for successor in block.successors:
                    if successor in function_blocks:
                        cfg.graph.add_edge(block, successor)

            cfgs[function_name] = cfg
            logger.info(
                f"Built CFG for function {function_name}: {len(function_blocks)} blocks")

        return cfgs

    def _find_function_blocks(self, entry_block: BasicBlock) -> Set[BasicBlock]:
        visited = set()
        stack = [entry_block]

        while stack:
            block = stack.pop()
            if block in visited:
                continue

            visited.add(block)

            for successor in block.successors:
                is_other_entry = any(
                    ep.address == successor.start_address
                    for ep in self.program.entry_points
                    if ep.address != entry_block.start_address
                )

                if not is_other_entry and successor not in visited:
                    stack.append(successor)

        return visited

    @property
    def cfgs(self) -> Dict[str, ControlFlowGraph]:
        return self._cfgs
