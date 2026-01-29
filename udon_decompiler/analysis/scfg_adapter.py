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
        for block in self.cfg.graph.nodes():
            block: BasicBlock
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
                ordered.append(false_block)
            if true_block and true_block in succs and true_block not in ordered:
                ordered.append(true_block)
            for s in sorted(succs, key=lambda b: b.start_address):
                if s not in ordered:
                    ordered.append(s)
            # false, true, others
            return [self._block_name(s) for s in ordered]
        return [
            self._block_name(s) for s in sorted(succs, key=lambda b: b.start_address)
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
