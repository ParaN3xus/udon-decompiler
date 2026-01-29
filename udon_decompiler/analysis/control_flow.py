from dataclasses import dataclass, field
from enum import Enum
from typing import List, Optional, Set

from udon_decompiler.analysis.basic_block import BasicBlock, BasicBlockType
from udon_decompiler.analysis.cfg import ControlFlowGraph
from udon_decompiler.models.instruction import OpCode
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
    # For loops, the block that contains the loop condition (if any).
    condition_block: Optional[BasicBlock] = None

    true_branch: List[BasicBlock] = field(default_factory=list)
    false_branch: Optional[List[BasicBlock]] = None

    loop_body: Optional[List[BasicBlock]] = None

    @property
    def body(self) -> List[BasicBlock]:
        result = [self.header]

        if self.type in (ControlStructureType.IF, ControlStructureType.IF_ELSE):
            result.extend(self.true_branch)
            if self.false_branch:
                result.extend(self.false_branch)
        elif self.type in (ControlStructureType.WHILE, ControlStructureType.DO_WHILE):
            if self.loop_body:
                result.extend(self.loop_body)

        result.sort()
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
        logger.debug(f"Identifying control structures for {self.cfg.function_name}...")

        loops = self._identify_loops()
        self._structures.extend(loops)
        logger.debug(f"Found {len(loops)} loops")

        conditionals = self._identify_conditionals()
        self._structures.extend(conditionals)
        logger.debug(f"Found {len(conditionals)} conditional structures")

        return self._structures

    def _identify_loops(self) -> List[ControlStructure]:
        loops = []
        detected_loop_blocks = self.cfg.get_loops()

        for loop_blocks in detected_loop_blocks:
            header = self._find_loop_header(loop_blocks)
            if header is None:
                continue

            loop_type = self._determine_loop_type(header, loop_blocks)
            exit_block = self._find_loop_exit(loop_blocks)
            condition_block = None
            if loop_type == ControlStructureType.WHILE:
                condition_block = (
                    header if header.block_type == BasicBlockType.CONDITIONAL else None
                )
            elif loop_type == ControlStructureType.DO_WHILE:
                condition_block = self._find_do_while_condition_block(
                    loop_blocks, exit_block
                )

            body_set = loop_blocks - {header}

            loop_body_list = list(body_set)
            loop_body_list.sort()

            structure = ControlStructure(
                type=loop_type,
                header=header,
                exit=exit_block,
                condition_block=condition_block,
                loop_body=loop_body_list,
            )
            loops.append(structure)

        return loops

    def _find_loop_exit(self, loop_blocks: Set[BasicBlock]) -> Optional[BasicBlock]:
        for block in loop_blocks:
            for succ in self.cfg.get_successors(block):
                if succ not in loop_blocks:
                    return succ
        return None

    def _find_do_while_condition_block(
        self, loop_blocks: Set[BasicBlock], exit_block: Optional[BasicBlock]
    ) -> Optional[BasicBlock]:
        if exit_block is None:
            return None

        candidates = []
        for block in loop_blocks:
            if block.block_type != BasicBlockType.CONDITIONAL:
                continue
            succs = self.cfg.get_successors(block)
            if exit_block in succs and any(s in loop_blocks for s in succs):
                candidates.append(block)

        if not candidates:
            return None

        # Prefer the latest conditional in the loop (tail condition).
        return max(candidates)

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
        # post-tested
        if header.block_type != BasicBlockType.CONDITIONAL:
            return ControlStructureType.DO_WHILE

        successors = self.cfg.get_successors(header)

        inner_successors = []
        outer_successors = []

        for succ in successors:
            if succ in loop_blocks:
                inner_successors.append(succ)
            else:
                outer_successors.append(succ)

        # while -> jump outer in the header
        # do while -> jump outer at the end
        if not outer_successors:
            return ControlStructureType.DO_WHILE

        if not inner_successors:
            raise Exception("Invalid loop! inner_successors expected!")

        first_inner_succ = inner_successors[0]

        if self._is_pure_latch(first_inner_succ):
            return ControlStructureType.DO_WHILE

        return ControlStructureType.WHILE

    def _is_pure_latch(self, block: BasicBlock) -> bool:
        instructions = block.instructions

        if len(instructions) != 1:
            return False

        inst = instructions[0]
        return inst.opcode == OpCode.JUMP

    def _identify_conditionals(self) -> List[ControlStructure]:
        conditionals = []
        loop_condition_blocks = {
            s.condition_block
            for s in self._structures
            if s.type in (ControlStructureType.WHILE, ControlStructureType.DO_WHILE)
            and s.condition_block is not None
        }

        for block in self.cfg.graph.nodes():
            block: BasicBlock
            if block.block_type != BasicBlockType.CONDITIONAL:
                continue
            if block in loop_condition_blocks:
                continue

            successors = self.cfg.get_successors(block)

            # unconditional
            if len(successors) != 2:
                logger.warning(
                    "Invalid conditional block! More successors expected! Ignoring..."
                )
                continue

            structure = self._analyze_conditional_structure(block, successors)
            if structure:
                conditionals.append(structure)

        return conditionals

    def _analyze_conditional_structure(
        self, header: BasicBlock, successors: List[BasicBlock]
    ) -> Optional[ControlStructure]:
        branch_false = successors[0]
        branch_true = successors[1]

        merge_point = self._find_merge_point(header, branch_true, branch_false)

        if merge_point is None:
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

        def get_reflexive_pdoms(block: BasicBlock) -> set[BasicBlock]:
            pdoms = post_dominators.get(block, set()).copy()
            pdoms.add(block)
            return pdoms

        header_pdoms = get_reflexive_pdoms(header)
        true_pdoms = get_reflexive_pdoms(branch_true)
        false_pdoms = get_reflexive_pdoms(branch_false)

        common_candidates = header_pdoms & true_pdoms & false_pdoms

        if not common_candidates:
            return None

        best_candidate = None
        max_pdom_size = -1

        for candidate in common_candidates:
            candidate_pdoms = get_reflexive_pdoms(candidate)
            size = len(candidate_pdoms)

            if size > max_pdom_size:
                max_pdom_size = size
                best_candidate = candidate

        return best_candidate

    def _get_reachable_blocks_dfs(self, start: BasicBlock) -> List[BasicBlock]:
        visited = set()
        stack = [start]
        result_list = []

        while stack:
            block = stack.pop()
            if block in visited:
                continue

            visited.add(block)
            result_list.append(block)

            for succ in self.cfg.get_successors(block):
                if succ not in visited:
                    stack.append(succ)

        result_list.sort()
        return result_list

    def _collect_blocks_between(
        self, start: BasicBlock, end: Optional[BasicBlock]
    ) -> List[BasicBlock]:
        if end is None:
            return self._get_reachable_blocks_dfs(start)

        blocks_list = []
        stack = [start]
        visited = set()

        while stack:
            block = stack.pop()
            if block in visited or block == end:
                continue

            visited.add(block)
            blocks_list.append(block)

            for succ in self.cfg.get_successors(block):
                if succ not in visited and succ != end:
                    stack.append(succ)

        blocks_list.sort()
        return blocks_list

    def _collect_reachable_blocks(
        self, start: BasicBlock, exclude: Optional[BasicBlock]
    ) -> List[BasicBlock]:
        blocks = [start]
        stack = [start]
        visited = {start}

        while stack:
            block = stack.pop()

            for succ in self.cfg.get_successors(block):
                if succ == exclude or succ in visited:
                    continue

                visited.add(succ)
                blocks.append(succ)
                stack.append(succ)

        blocks.sort()
        return blocks

    @property
    def structures(self) -> List[ControlStructure]:
        return self._structures
