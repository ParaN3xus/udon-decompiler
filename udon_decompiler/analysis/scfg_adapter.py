from __future__ import annotations

from dataclasses import dataclass
from typing import Dict, List

from numba_scfg.core.datastructures.basic_block import (
    BasicBlock as SCFGBasicBlock,
)
from numba_scfg.core.datastructures.basic_block import (
    SyntheticBranch,
)
from numba_scfg.core.datastructures.scfg import SCFG

from udon_decompiler.analysis.basic_block import BasicBlock, BasicBlockType
from udon_decompiler.analysis.cfg import ControlFlowGraph
from udon_decompiler.models.instruction import OpCode
from udon_decompiler.utils.logger import logger


@dataclass
class SCFGBuildResult:
    scfg: SCFG
    name_to_block: Dict[str, BasicBlock]
    switch_branches: Dict[str, BasicBlock]


class SCFGAdapter:
    def __init__(self, cfg: ControlFlowGraph):
        self.cfg = cfg
        self._name_to_block: Dict[str, BasicBlock] = {}
        self._block_to_name: Dict[BasicBlock, str] = {}
        self._switch_branches: Dict[str, BasicBlock] = {}
        self._switch_counter = 0
        self._canonical_cache: Dict[BasicBlock, BasicBlock] = {}
        self._loop_blocks: set[BasicBlock] = set()

    def build(self) -> SCFGBuildResult:
        logger.debug(f"Building SCFG for {self.cfg.function_name}...")
        scfg = self._build_scfg()
        scfg.restructure()
        logger.debug(f"SCFG built for {self.cfg.function_name}")
        return SCFGBuildResult(
            scfg=scfg,
            name_to_block=self._name_to_block,
            switch_branches=self._switch_branches,
        )

    def _build_scfg(self) -> SCFG:
        graph: Dict[str, SCFGBasicBlock] = {}
        loops = self.cfg.get_loops()
        if loops:
            self._loop_blocks = set().union(*loops)
        for block in self.cfg.graph.nodes():
            block: BasicBlock
            canonical = self._canonical_block(block)
            if canonical is not block:
                continue
            name = self._block_name(block)
            if block.switch_info is not None:
                jump_targets, branch_value_table = self._build_switch_targets(block)
                switch_var = self._next_switch_var()
                graph[name] = SyntheticBranch(
                    name=name,
                    _jump_targets=tuple(jump_targets),
                    backedges=tuple(),
                    variable=switch_var,
                    branch_value_table=branch_value_table,
                )
                self._switch_branches[name] = block
            else:
                jump_targets = self._build_jump_targets(block)
                graph[name] = SCFGBasicBlock(
                    name=name,
                    _jump_targets=tuple(jump_targets),
                    backedges=tuple(),
                )
        return SCFG(graph=graph)

    def _block_name(self, block: BasicBlock) -> str:
        block = self._canonical_block(block)
        existing = self._block_to_name.get(block)
        if existing:
            return existing
        name = f"bb_{block.start_address:08x}"
        self._block_to_name[block] = name
        self._name_to_block[name] = block
        return name

    def _build_jump_targets(self, block: BasicBlock) -> List[str]:
        succs = list(self.cfg.get_successors(block))
        if not succs:
            return []
        if block.block_type == BasicBlockType.CONDITIONAL:
            false_target = block.last_instruction.get_jump_target()
            if false_target is not None:
                false_block = self.cfg.get_block_at(false_target)
            else:
                false_block = None
            true_block = self.cfg.get_block_at(block.last_instruction.next_address)
            ordered: List[BasicBlock] = []
            if false_block and false_block in succs:
                ordered.append(self._canonical_block(false_block))
            if true_block and true_block in succs and true_block not in ordered:
                ordered.append(self._canonical_block(true_block))
            for s in sorted(succs, key=lambda b: b.start_address):
                canonical = self._canonical_block(s)
                if canonical not in ordered:
                    ordered.append(canonical)
            # false, true, others
            return self._dedup_targets([self._block_name(s) for s in ordered])
        ordered = [
            self._canonical_block(s)
            for s in sorted(succs, key=lambda b: b.start_address)
        ]
        return [
            self._block_name(s) for s in self._dedup_blocks(ordered)
        ]

    def _build_switch_targets(
        self, block: BasicBlock
    ) -> tuple[List[str], Dict[int, str]]:
        switch_info = block.switch_info
        if switch_info is None:
            return [], {}
        branch_value_table: Dict[int, str] = {}
        ordered_targets: List[str] = []
        seen: set[str] = set()
        for idx, addr in enumerate(switch_info.targets):
            target_block = self.cfg.get_block_at(addr)
            if target_block is None:
                continue
            target_block = self._canonical_block(target_block)
            target_name = self._block_name(target_block)
            branch_value_table[idx] = target_name
            if target_name not in seen:
                seen.add(target_name)
                ordered_targets.append(target_name)
        return ordered_targets, branch_value_table

    def _next_switch_var(self) -> str:
        name = f"__scfg_switch_control_{self._switch_counter}__"
        self._switch_counter += 1
        return name

    def _canonical_block(self, block: BasicBlock) -> BasicBlock:
        cached = self._canonical_cache.get(block)
        if cached is not None:
            return cached
        seen: set[BasicBlock] = set()
        current = block
        while True:
            if current in seen:
                break
            seen.add(current)
            if not self._is_trampoline_block(current):
                break
            succs = list(self.cfg.get_successors(current))
            if len(succs) != 1:
                break
            current = succs[0]
        for b in seen:
            self._canonical_cache[b] = current
        return current

    def _is_trampoline_block(self, block: BasicBlock) -> bool:
        if block in self._loop_blocks:
            return False
        if block.switch_info is not None:
            return False
        if block.block_type != BasicBlockType.JUMP:
            return False
        if len(block.instructions) != 1:
            return False
        return block.last_instruction.opcode == OpCode.JUMP

    def _dedup_targets(self, targets: List[str]) -> List[str]:
        seen: set[str] = set()
        ordered: List[str] = []
        for name in targets:
            if name in seen:
                continue
            seen.add(name)
            ordered.append(name)
        return ordered

    def _dedup_blocks(self, blocks: List[BasicBlock]) -> List[BasicBlock]:
        seen: set[BasicBlock] = set()
        ordered: List[BasicBlock] = []
        for block in blocks:
            if block in seen:
                continue
            seen.add(block)
            ordered.append(block)
        return ordered
