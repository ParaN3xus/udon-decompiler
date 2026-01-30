from __future__ import annotations

from typing import List, Optional

from udon_decompiler.analysis.expression_builder import Operator
from udon_decompiler.codegen.ast_nodes import (
    AssignmentNode,
    BlockNode,
    BreakNode,
    DoWhileNode,
    IfElseNode,
    IfNode,
    LiteralNode,
    OperatorNode,
    StatementNode,
    SwitchNode,
    VariableNode,
    WhileNode,
)


class BackedgeLifter:
    def lift(self, statements: List[StatementNode]) -> List[StatementNode]:
        block = BlockNode(statements=statements)
        self._lift_block(block)
        return block.statements

    def _lift_block(self, block: BlockNode) -> None:
        i = 0
        while i < len(block.statements):
            stmt = block.statements[i]
            if isinstance(stmt, WhileNode):
                if stmt.body:
                    self._lift_block(stmt.body)
                self._try_lift_backedge(stmt)
            elif isinstance(stmt, DoWhileNode):
                if stmt.body:
                    self._lift_block(stmt.body)
            elif isinstance(stmt, IfNode):
                if stmt.then_block:
                    self._lift_block(stmt.then_block)
            elif isinstance(stmt, IfElseNode):
                if stmt.then_block:
                    self._lift_block(stmt.then_block)
                if stmt.else_block:
                    self._lift_block(stmt.else_block)
            elif isinstance(stmt, SwitchNode):
                for case in stmt.cases:
                    if case.body:
                        self._lift_block(case.body)
                if stmt.default_case and stmt.default_case.body:
                    self._lift_block(stmt.default_case.body)
            i += 1

    def _try_lift_backedge(self, loop: WhileNode) -> None:
        if not loop.body or not loop.body.statements:
            return
        last = loop.body.statements[-1]
        backedge_var, break_value = self._parse_backedge_break(loop, last)
        if backedge_var is None or break_value is None:
            return
        inserted = self._rewrite_backedge_assignments(
            loop.body, backedge_var, break_value
        )
        if inserted:
            loop.body.statements.pop()

    def _parse_backedge_break(
        self, loop: WhileNode, stmt: StatementNode
    ) -> tuple[Optional[str], Optional[int]]:
        if loop.body is None:
            return None, None
        if not isinstance(stmt, IfNode):
            return None, None
        if not stmt.condition or not stmt.then_block:
            return None, None
        if not stmt.then_block.statements:
            return None, None
        if not isinstance(stmt.then_block.statements[-1], BreakNode):
            return None, None
        cond = stmt.condition
        if not isinstance(cond, OperatorNode):
            return None, None
        if cond.operator not in (Operator.Equality, Operator.Inequality):
            return None, None
        if len(cond.operands) != 2:
            return None, None
        lhs, rhs = cond.operands
        if not isinstance(lhs, VariableNode) or not isinstance(rhs, LiteralNode):
            return None, None
        if not isinstance(rhs.value, int):
            return None, None
        if cond.operator == Operator.Equality:
            return lhs.var_name, rhs.value

        assigned_values = self._collect_assigned_values(loop.body, lhs.var_name)
        if rhs.value in assigned_values and len(assigned_values) == 2:
            assigned_values.remove(rhs.value)
            return lhs.var_name, assigned_values.pop()
        return None, None

    def _rewrite_backedge_assignments(
        self, block: BlockNode, backedge_var: str, break_value: int
    ) -> bool:
        inserted_break = False
        i = 0
        while i < len(block.statements):
            stmt = block.statements[i]
            if isinstance(stmt, (WhileNode, DoWhileNode)):
                i += 1
                continue
            if isinstance(stmt, IfNode):
                if stmt.then_block:
                    inserted_break |= self._rewrite_backedge_assignments(
                        stmt.then_block, backedge_var, break_value
                    )
                i += 1
                continue
            if isinstance(stmt, IfElseNode):
                if stmt.then_block:
                    inserted_break |= self._rewrite_backedge_assignments(
                        stmt.then_block, backedge_var, break_value
                    )
                if stmt.else_block:
                    inserted_break |= self._rewrite_backedge_assignments(
                        stmt.else_block, backedge_var, break_value
                    )
                i += 1
                continue
            if isinstance(stmt, AssignmentNode) and stmt.target == backedge_var:
                value = stmt.value
                if not isinstance(value, LiteralNode) or not isinstance(
                    value.value, int
                ):
                    i += 1
                    continue
                if value.value == break_value:
                    block.statements[i] = BreakNode()
                    inserted_break = True
                else:
                    block.statements.pop(i)
                    continue
            i += 1
        return inserted_break

    def _collect_assigned_values(
        self, block: BlockNode, backedge_var: str
    ) -> set[int]:
        values: set[int] = set()
        self._collect_assigned_values_in_block(block, backedge_var, values)
        return values

    def _collect_assigned_values_in_block(
        self, block: BlockNode, backedge_var: str, values: set[int]
    ) -> None:
        for stmt in block.statements:
            if isinstance(stmt, (WhileNode, DoWhileNode)):
                continue
            if isinstance(stmt, AssignmentNode) and stmt.target == backedge_var:
                if isinstance(stmt.value, LiteralNode) and isinstance(
                    stmt.value.value, int
                ):
                    values.add(stmt.value.value)
            if isinstance(stmt, IfNode):
                if stmt.then_block:
                    self._collect_assigned_values_in_block(
                        stmt.then_block, backedge_var, values
                    )
            if isinstance(stmt, IfElseNode):
                if stmt.then_block:
                    self._collect_assigned_values_in_block(
                        stmt.then_block, backedge_var, values
                    )
                if stmt.else_block:
                    self._collect_assigned_values_in_block(
                        stmt.else_block, backedge_var, values
                    )
