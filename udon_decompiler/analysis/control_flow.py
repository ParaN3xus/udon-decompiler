from dataclasses import dataclass, field
from enum import Enum
from typing import List, Optional, Set

from udon_decompiler.analysis.basic_block import BasicBlock, BasicBlockType
from udon_decompiler.analysis.cfg import ControlFlowGraph
from udon_decompiler.utils.logger import logger


class ControlStructureType(Enum):
    IF = "if"
    IF_ELSE = "if_else"
    WHILE = "while"
    DO_WHILE = "do_while"
    # FOR = "for"
    # SWITCH_CASE = "switch_case"


@dataclass
class ControlStructure:
    type: ControlStructureType
    header: BasicBlock
    exit: Optional[BasicBlock] = None

    true_branch: Set[BasicBlock] = field(default_factory=set)
    false_branch: Optional[Set[BasicBlock]] = None

    loop_body: Optional[Set[BasicBlock]] = None

    @property
    def body(self) -> Set[BasicBlock]:
        result = {self.header}

        if self.type in (ControlStructureType.IF, ControlStructureType.IF_ELSE):
            result |= self.true_branch
            if self.false_branch:
                result |= self.false_branch
        elif self.type in (ControlStructureType.WHILE, ControlStructureType.DO_WHILE):
            if self.loop_body:
                result |= self.loop_body

        return result

    def __repr__(self) -> str:
        return (
            f"ControlStructure(type={self.type.value}, "
            f"header=0x{self.header.start_address:08x}, "
            f"body_size={len(self.body)})"
        )


class ControlFlowStructureIdentifier:
    def __init__(self, cfg: ControlFlowGraph):
        self.cfg = cfg
        self._structures: List[ControlStructure] = []

    def identify(self) -> List[ControlStructure]:
        logger.info(f"Identifying control structures for {self.cfg.function_name}...")

        loops = self._identify_loops()
        self._structures.extend(loops)
        logger.info(f"Found {len(loops)} loops")

        conditionals = self._identify_conditionals()
        self._structures.extend(conditionals)
        logger.info(f"Found {len(conditionals)} conditional structures")

        return self._structures

    def _identify_loops(self) -> List[ControlStructure]:
        loops = []
        detected_loop_blocks = self.cfg.get_loops()

        for loop_blocks in detected_loop_blocks:
            header = self._find_loop_header(loop_blocks)
            if not header:
                continue

            loop_type = self._determine_loop_type(header, loop_blocks)

            exit_block = self._find_loop_exit(loop_blocks)

            structure = ControlStructure(
                type=loop_type,
                header=header,
                exit=exit_block,
                loop_body=loop_blocks - {header},
            )
            loops.append(structure)

        return loops

    def _find_loop_exit(self, loop_blocks: Set[BasicBlock]) -> Optional[BasicBlock]:
        for block in loop_blocks:
            for succ in self.cfg.get_successors(block):
                if succ not in loop_blocks:
                    return succ
        return None

    def _find_loop_header(self, loop_blocks: Set[BasicBlock]) -> Optional[BasicBlock]:
        """
        have predecessors outside the loop
        have predecessors inside the loop
        min addr
        """
        dominators = self.cfg.get_dominators()
        candidates = []

        for block in loop_blocks:
            preds = self.cfg.get_predecessors(block)

            external_preds = [p for p in preds if p not in loop_blocks]
            internal_preds = [p for p in preds if p in loop_blocks]

            if external_preds and internal_preds:
                block_doms = dominators.get(block, set())
                if loop_blocks - {block} <= block_doms:
                    candidates.append(block)

        if candidates:
            return min(candidates, key=lambda b: b.start_address)

        # incase, downgraded
        blocks_with_external_pred = [
            b
            for b in loop_blocks
            if any(p not in loop_blocks for p in self.cfg.get_predecessors(b))
        ]

        return (
            min(blocks_with_external_pred, key=lambda b: b.start_address)
            if blocks_with_external_pred
            else None
        )

    def _determine_loop_type(
        self, header: BasicBlock, loop_blocks: Set[BasicBlock]
    ) -> ControlStructureType:
        external_preds = [
            pred
            for pred in self.cfg.get_predecessors(header)
            if pred not in loop_blocks
        ]

        if external_preds and header.block_type == BasicBlockType.CONDITIONAL:
            return ControlStructureType.WHILE
        else:
            return ControlStructureType.DO_WHILE

    def _identify_conditionals(self) -> List[ControlStructure]:
        conditionals = []

        for block in self.cfg.graph.nodes():
            if block.block_type != BasicBlockType.CONDITIONAL:
                continue

            successors = self.cfg.get_successors(block)

            # unconditional
            if len(successors) != 2:
                continue

            structure = self._analyze_conditional_structure(block, successors)
            if structure:
                conditionals.append(structure)

        return conditionals

    def _analyze_conditional_structure(
        self, header: BasicBlock, successors: List[BasicBlock]
    ) -> Optional[ControlStructure]:
        branch_true = successors[0]
        branch_false = successors[1]

        merge_point = self._find_merge_point(header, branch_true, branch_false)

        if not merge_point:
            # if xxx { return a; } else { return b; }
            true_branch = self._collect_reachable_blocks(branch_true, None)
            false_branch = self._collect_reachable_blocks(branch_false, None)

            return ControlStructure(
                type=ControlStructureType.IF_ELSE,
                header=header,
                exit=None,
                true_branch=true_branch,
                false_branch=false_branch,
            )

        else:
            true_blocks = self._collect_blocks_between(branch_true, merge_point)
            false_blocks = self._collect_blocks_between(branch_false, merge_point)

            if branch_false == merge_point or not false_blocks:
                return ControlStructure(
                    type=ControlStructureType.IF,
                    header=header,
                    exit=merge_point,
                    true_branch=true_blocks,
                    false_branch=None,
                )
            else:
                return ControlStructure(
                    type=ControlStructureType.IF_ELSE,
                    header=header,
                    exit=merge_point,
                    true_branch=true_blocks,
                    false_branch=false_blocks,
                )

    def _find_merge_point(
        self, header: BasicBlock, branch_true: BasicBlock, branch_false: BasicBlock
    ) -> Optional[BasicBlock]:
        post_dominators = self.cfg.get_post_dominators()

        header_post_doms = post_dominators.get(header, set())

        if not header_post_doms:
            return None

        true_post_doms = post_dominators.get(branch_true, set())
        false_post_doms = post_dominators.get(branch_false, set())

        common_post_doms = header_post_doms & true_post_doms & false_post_doms

        if not common_post_doms:
            return None

        candidates = common_post_doms.copy()
        for pd1 in common_post_doms:
            for pd2 in common_post_doms:
                if pd1 != pd2 and pd2 in post_dominators.get(pd1, set()):
                    candidates.discard(pd1)
                    break

        return candidates.pop() if candidates else None

    def _get_reachable_blocks_dfs(self, start: BasicBlock) -> Set[BasicBlock]:
        visited = set()
        stack = [start]

        while stack:
            block = stack.pop()
            if block in visited:
                continue

            visited.add(block)
            for succ in self.cfg.get_successors(block):
                if succ not in visited:
                    stack.append(succ)

        return visited

    def _collect_blocks_between(
        self, start: BasicBlock, end: Optional[BasicBlock]
    ) -> Set[BasicBlock]:
        if end is None:
            return self._get_reachable_blocks_dfs(start)

        blocks = set()
        stack = [start]
        visited = set()

        while stack:
            block = stack.pop()
            if block in visited or block == end:
                continue

            visited.add(block)
            blocks.add(block)

            for succ in self.cfg.get_successors(block):
                if succ not in visited and succ != end:
                    stack.append(succ)

        return blocks

    def _collect_reachable_blocks(
        self, start: BasicBlock, exclude: Optional[BasicBlock]
    ) -> Set[BasicBlock]:
        blocks = {start}
        stack = [start]
        visited = {start}

        while stack:
            block = stack.pop()

            for succ in self.cfg.get_successors(block):
                if succ == exclude or succ in visited:
                    continue

                visited.add(succ)
                blocks.add(succ)
                stack.append(succ)

        return blocks

    @property
    def structures(self) -> List[ControlStructure]:
        return self._structures
