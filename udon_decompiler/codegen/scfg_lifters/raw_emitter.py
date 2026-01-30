from __future__ import annotations

from typing import TYPE_CHECKING, Dict, List, Optional, Set

from numba_scfg.core.datastructures.basic_block import (
    BasicBlock as SCFGBasicBlock,
)
from numba_scfg.core.datastructures.basic_block import (
    RegionBlock,
    SyntheticAssignment,
    SyntheticBranch,
    SyntheticExitBranch,
    SyntheticExitingLatch,
    SyntheticFill,
    SyntheticHead,
    SyntheticReturn,
    SyntheticTail,
)
from numba_scfg.core.datastructures.scfg import SCFG

from udon_decompiler.analysis.basic_block import BasicBlock as CFGBlock
from udon_decompiler.analysis.basic_block import BasicBlockType
from udon_decompiler.analysis.expression_builder import Operator
from udon_decompiler.codegen.ast_nodes import (
    AssignmentNode,
    BlockNode,
    BreakNode,
    ExpressionNode,
    ExpressionStatementNode,
    IfElseNode,
    IfNode,
    LiteralNode,
    OperatorNode,
    StatementNode,
    SwitchCaseNode,
    SwitchNode,
    VariableDeclNode,
    VariableNode,
    WhileNode,
)

if TYPE_CHECKING:
    from udon_decompiler.codegen.ast_builder import ASTBuilder


class SCFGRawEmitter:
    def __init__(
        self, ast_builder: ASTBuilder, scfg: SCFG, name_to_block, switch_branches
    ):
        self._builder = ast_builder
        self._scfg = scfg
        self._name_to_block: Dict[str, CFGBlock] = name_to_block
        self._switch_branches: Dict[str, CFGBlock] = switch_branches

        self._loop_stack: list[str] = []
        self._loop_counter = 0

        self._synthetic_decl_types: Dict[str, str] = {}
        self._scfg_stack: list[SCFG] = []
        self._emitted_by_scfg: Dict[int, Set[str]] = {}

    def emit(self) -> tuple[List[StatementNode], List[VariableDeclNode]]:
        statements: List[StatementNode] = self._emit_region(self._scfg.region)
        synthetic_decls = [
            VariableDeclNode(var_name=name, var_type=type_name)
            for name, type_name in sorted(self._synthetic_decl_types.items())
        ]
        return statements, synthetic_decls

    def _emit_region(self, region: RegionBlock) -> List[StatementNode]:
        assert region.subregion is not None
        if region.kind == "loop":
            loop_cont = self._new_loop_cont()
            self._declare_synthetic(loop_cont, "System.Boolean")
            self._loop_stack.append(loop_cont)

            body_statements = self._emit_sequence(region.subregion)
            loop_body = BlockNode(statements=body_statements)
            condition = VariableNode(var_name=loop_cont, var_type="System.Boolean")

            self._loop_stack.pop()

            return [
                AssignmentNode(
                    target=loop_cont,
                    value=LiteralNode(value=True, literal_type="System.Boolean"),
                ),
                WhileNode(condition=condition, body=loop_body),
            ]

        return self._emit_sequence(region.subregion)

    def _emit_sequence(self, scfg: SCFG) -> List[StatementNode]:
        emitted = self._emitted_by_scfg.setdefault(id(scfg), set())
        statements: List[StatementNode] = []
        self._scfg_stack.append(scfg)
        try:
            for name, block in scfg.concealed_region_view.items():
                if name in emitted:
                    continue
                if isinstance(block, RegionBlock) and block.kind == "branch":
                    continue
                emitted.add(name)
                statements.extend(self._emit_block(block))
            return statements
        finally:
            self._scfg_stack.pop()

    def _emit_block(self, block: SCFGBasicBlock) -> List[StatementNode]:
        if isinstance(block, RegionBlock):
            return self._emit_region(block)

        if isinstance(block, SyntheticAssignment):
            return self._emit_synthetic_assignment(block)

        if isinstance(block, SyntheticExitingLatch):
            return self._emit_exiting_latch(block)

        if isinstance(block, (SyntheticHead, SyntheticExitBranch, SyntheticBranch)):
            return self._emit_synthetic_branch(block)

        if isinstance(block, SyntheticTail):
            return []

        if isinstance(block, SyntheticFill):
            return [ExpressionStatementNode(expression=None)]

        if isinstance(block, SyntheticReturn):
            return []

        # Regular SCFG basic block mapped from CFG.
        return self._emit_cfg_block(block)

    def _emit_cfg_block(self, block: SCFGBasicBlock) -> List[StatementNode]:
        cfg_block = self._name_to_block.get(block.name)
        if cfg_block is None:
            return []

        # switch branch blocks are handled via SyntheticBranch.
        if block.name in self._switch_branches:
            return []

        if (
            cfg_block.block_type == BasicBlockType.CONDITIONAL
            and len(block.jump_targets) >= 2
        ):
            statements = self._builder._translate_block_for_condition(cfg_block)
            condition = self._builder._extract_condition_from_block(cfg_block)
            false_target = block.jump_targets[0]
            true_target = block.jump_targets[1]
            then_block = self._emit_target(true_target)
            else_block = self._emit_target(false_target)

            if else_block and else_block.statements:
                statements.append(
                    IfElseNode(
                        condition=condition,
                        then_block=then_block,
                        else_block=else_block,
                    )
                )
            else:
                statements.append(IfNode(condition=condition, then_block=then_block))
            return statements

        block_node = BlockNode()
        self._builder._translate_basic_block(cfg_block, block_node)
        return block_node.statements

    def _emit_target(self, name: str) -> BlockNode:
        block, scfg = self._resolve_block(name)
        if block is None or scfg is None:
            return BlockNode()
        emitted = self._emitted_by_scfg.setdefault(id(scfg), set())
        if name in emitted:
            return BlockNode()
        emitted.add(name)
        if isinstance(block, RegionBlock):
            return BlockNode(statements=self._emit_region(block))
        return BlockNode(statements=self._emit_block(block))

    def _emit_synthetic_assignment(
        self, block: SyntheticAssignment
    ) -> List[StatementNode]:
        statements: List[StatementNode] = []
        for target, value in block.variable_assignment.items():
            type_hint = "System.Int32"
            if isinstance(value, bool):
                type_hint = "System.Boolean"
            self._declare_synthetic(target, type_hint)
            statements.append(
                AssignmentNode(
                    target=target,
                    value=LiteralNode(value=value, literal_type=type_hint),
                )
            )
        return statements

    def _emit_exiting_latch(self, block: SyntheticExitingLatch) -> List[StatementNode]:
        if not self._loop_stack:
            return []
        loop_cont = self._loop_stack[-1]
        self._declare_synthetic(loop_cont, "System.Boolean")
        self._declare_synthetic(block.variable, "System.Int32")
        condition = OperatorNode(
            operator=Operator.Equality,
            receiver=None,
            operands=[
                VariableNode(var_name=block.variable, var_type="System.Int32"),
                LiteralNode(value=0, literal_type="System.Int32"),
            ],
        )
        return [AssignmentNode(target=loop_cont, value=condition)]

    def _emit_synthetic_branch(self, block: SyntheticBranch) -> List[StatementNode]:
        # switch branch created from actual switch.
        if block.name in self._switch_branches:
            return self._emit_switch_branch(block)

        self._declare_synthetic(block.variable, "System.Int32")

        # reverse lookup: target -> list of values.
        reverse: Dict[str, List[int]] = {}
        for value, target in block.branch_value_table.items():
            reverse.setdefault(target, []).append(value)

        jump_targets = list(block.jump_targets)
        if not jump_targets:
            return []
        if len(jump_targets) == 1:
            target_block, _ = self._resolve_block(jump_targets[0])
            if target_block is None:
                return []
            return self._emit_block(target_block)

        # emit if-cascade
        def cascade(targets: List[str]) -> List[StatementNode]:
            if len(targets) == 1:
                return self._emit_target(targets[0]).statements

            current = targets.pop()
            values = reverse.get(current, [])
            if not values:
                return self._emit_target(current).statements

            if len(values) == 1:
                cond = OperatorNode(
                    operator=Operator.Equality,
                    receiver=None,
                    operands=[
                        VariableNode(var_name=block.variable, var_type="System.Int32"),
                        LiteralNode(value=values[0], literal_type="System.Int32"),
                    ],
                )
            else:
                # collapse multiple values to nested ORs.
                cond = None
                for value in values:
                    cmp = OperatorNode(
                        operator=Operator.Equality,
                        receiver=None,
                        operands=[
                            VariableNode(
                                var_name=block.variable, var_type="System.Int32"
                            ),
                            LiteralNode(value=value, literal_type="System.Int32"),
                        ],
                    )
                    if cond is None:
                        cond = cmp
                    else:
                        cond = OperatorNode(
                            operator=Operator.LogicalOr,
                            receiver=None,
                            operands=[cond, cmp],
                        )
                assert cond is not None

            then_block = self._emit_target(current)
            else_block = BlockNode(statements=cascade(targets))
            return [
                IfElseNode(condition=cond, then_block=then_block, else_block=else_block)
            ]

        return cascade(list(jump_targets[::-1]))

    def _emit_switch_branch(self, block: SyntheticBranch) -> List[StatementNode]:
        cfg_block = self._switch_branches.get(block.name)
        if cfg_block is None or cfg_block.switch_info is None:
            return []
        switch_info = cfg_block.switch_info
        switch_var = self._builder.analyzer.variable_identifier.get_variable(
            switch_info.index_operand
        )
        if switch_var is None:
            switch_expr: ExpressionNode = LiteralNode(
                value="<switch>", literal_type="System.Int32"
            )
        else:
            switch_expr = self._builder._variable_to_ast(switch_var)

        # Group case values by target.
        values_by_target: Dict[str, List[int]] = {}
        for value, target in block.branch_value_table.items():
            values_by_target.setdefault(target, []).append(value)

        cases: List[SwitchCaseNode] = []
        for target, values in values_by_target.items():
            body_block = self._emit_target(target)
            case_values: List[ExpressionNode] = [
                LiteralNode(value=v, literal_type="System.Int32") for v in values
            ]
            cases.append(SwitchCaseNode(values=case_values, body=body_block))

        return [SwitchNode(expression=switch_expr, cases=cases)]

    def _new_loop_cont(self) -> str:
        self._loop_counter += 1
        return f"__scfg_loop_cont_{self._loop_counter}__"

    def _declare_synthetic(self, name: str, type_hint: str) -> None:
        if name not in self._synthetic_decl_types:
            self._synthetic_decl_types[name] = type_hint

    def _resolve_block(
        self, name: str
    ) -> tuple[Optional[SCFGBasicBlock], Optional[SCFG]]:
        for scfg in reversed(self._scfg_stack):
            block = scfg.graph.get(name)
            if block is not None:
                return block, scfg
        # Fallback to top-level SCFG.
        block = self._scfg.graph.get(name)
        if block is not None:
            return block, self._scfg
        return None, None
