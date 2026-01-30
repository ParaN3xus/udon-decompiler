from __future__ import annotations

from dataclasses import dataclass
from typing import Dict, List, Optional

from udon_decompiler.analysis.expression_builder import Operator
from udon_decompiler.codegen.ast_nodes import (
    AssignmentNode,
    BlockNode,
    BreakNode,
    ContinueNode,
    DoWhileNode,
    ExpressionNode,
    ExpressionStatementNode,
    IfElseNode,
    IfNode,
    LiteralNode,
    OperatorNode,
    StatementNode,
    SwitchCaseNode,
    SwitchNode,
    VariableNode,
    WhileNode,
)


@dataclass
class _ExitSite:
    block: BlockNode
    break_index: int


class ExitVarLifter:
    def lift(self, statements: List[StatementNode]) -> List[StatementNode]:
        block = BlockNode(statements=statements)
        self._lift_block(block)
        return block.statements

    def _lift_block(self, block: BlockNode) -> None:
        i = 0
        while i < len(block.statements):
            stmt = block.statements[i]
            if isinstance(stmt, (WhileNode, DoWhileNode)):
                if stmt.body:
                    self._lift_block(stmt.body)
                if i + 1 < len(block.statements):
                    next_stmt = block.statements[i + 1]
                    if (
                        isinstance(next_stmt, SwitchNode)
                        and self._try_inline_switch_after_loop(stmt, next_stmt)
                    ):
                        block.statements.pop(i + 1)
                        continue
                if i + 1 < len(block.statements):
                    next_stmt = block.statements[i + 1]
                    if (
                        isinstance(next_stmt, IfNode)
                        and self._try_inline_if_after_loop(stmt, next_stmt)
                    ):
                        block.statements.pop(i + 1)
                        continue
                if i + 1 < len(block.statements):
                    next_stmt = block.statements[i + 1]
                    if (
                        isinstance(next_stmt, IfElseNode)
                        and self._try_inline_if_after_loop(stmt, next_stmt)
                    ):
                        block.statements.pop(i + 1)
                        continue
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

    def _try_inline_switch_after_loop(
        self, loop: WhileNode | DoWhileNode, switch: SwitchNode
    ) -> bool:
        if not isinstance(switch.expression, VariableNode):
            return False
        exit_var = switch.expression.var_name
        if exit_var is None:
            return False

        if not loop.body:
            return False

        exit_sites = self._collect_exit_sites(loop.body, exit_var)
        if not exit_sites:
            return False

        if switch.default_case and not self._case_is_trivial(switch.default_case):
            return False

        for case in switch.cases:
            if not self._case_is_trivial(case):
                return False

        # Inline case bodies into corresponding break sites.
        for case in switch.cases:
            if not case.values:
                continue
            body_stmts = case.body.statements if case.body else []
            body_stmts = self._strip_trailing_break(body_stmts)
            for value_expr in case.values:
                literal = self._literal_int(value_expr)
                if literal is None:
                    return False
                sites = exit_sites.get(literal)
                if not sites:
                    return False
                for site in sites:
                    self._insert_before_break(site, body_stmts)

        return True

    def _try_inline_if_after_loop(
        self, loop: WhileNode | DoWhileNode, cond_if: IfNode | IfElseNode
    ) -> bool:
        if not isinstance(cond_if.condition, ExpressionNode):
            return False
        exit_var, value = self._match_exit_var_condition(cond_if.condition)
        if exit_var is None or value is None:
            return False
        if cond_if.then_block is None:
            return False
        if isinstance(cond_if, IfElseNode) and not self._block_is_effectively_empty(
            cond_if.else_block
        ):
            return False
        if not loop.body:
            return False
        inserted = self._inline_exit_if_simple(
            loop.body, exit_var, value, cond_if.then_block.statements
        )
        if inserted:
            return True
        return self._inline_exit_if(
            loop.body, exit_var, value, cond_if.then_block.statements
        )

    def _block_is_effectively_empty(self, block: Optional[BlockNode]) -> bool:
        if block is None or not block.statements:
            return True
        for stmt in block.statements:
            if isinstance(stmt, ExpressionStatementNode) and stmt.expression is None:
                continue
            return False
        return True

    def _collect_exit_sites(
        self, block: BlockNode, exit_var: str
    ) -> Dict[int, List[_ExitSite]]:
        sites: Dict[int, List[_ExitSite]] = {}
        self._collect_exit_sites_in_block(block, exit_var, sites)
        return sites

    def _collect_exit_sites_in_block(
        self,
        block: BlockNode,
        exit_var: str,
        sites: Dict[int, List[_ExitSite]],
    ) -> None:
        i = 0
        pending_value: Optional[int] = None
        while i < len(block.statements):
            stmt = block.statements[i]
            if isinstance(stmt, (WhileNode, DoWhileNode, SwitchNode)):
                i += 1
                continue
            if isinstance(stmt, IfNode):
                if stmt.then_block:
                    self._collect_exit_sites_in_block(
                        stmt.then_block, exit_var, sites
                    )
                i += 1
                continue
            if isinstance(stmt, IfElseNode):
                if stmt.then_block:
                    self._collect_exit_sites_in_block(
                        stmt.then_block, exit_var, sites
                    )
                if stmt.else_block:
                    self._collect_exit_sites_in_block(
                        stmt.else_block, exit_var, sites
                    )
                i += 1
                continue

            if isinstance(stmt, AssignmentNode) and stmt.target == exit_var:
                literal = self._literal_int(stmt.value)
                if literal is not None:
                    pending_value = literal
                i += 1
                continue
            if isinstance(stmt, ContinueNode):
                pending_value = None
                i += 1
                continue
            if isinstance(stmt, BreakNode) and pending_value is not None:
                sites.setdefault(pending_value, []).append(
                    _ExitSite(block=block, break_index=i)
                )
                if i > 0:
                    prev_stmt = block.statements[i - 1]
                    if (
                        isinstance(prev_stmt, AssignmentNode)
                        and prev_stmt.target == exit_var
                    ):
                        block.statements.pop(i - 1)
                        i -= 1
                pending_value = None
            i += 1

    def _case_is_trivial(self, case: SwitchCaseNode) -> bool:
        if case.body is None or not case.body.statements:
            return True
        for stmt in case.body.statements:
            if isinstance(stmt, BreakNode):
                continue
            if isinstance(stmt, (WhileNode, DoWhileNode, SwitchNode)):
                return False
        return True

    def _match_exit_var_condition(
        self, expr: ExpressionNode
    ) -> tuple[Optional[str], Optional[int]]:
        if not isinstance(expr, OperatorNode):
            return None, None
        if expr.operator not in (Operator.Equality, Operator.Inequality):
            return None, None
        if len(expr.operands) != 2:
            return None, None
        lhs, rhs = expr.operands
        if isinstance(lhs, LiteralNode) and isinstance(rhs, VariableNode):
            lhs, rhs = rhs, lhs
        if not isinstance(lhs, VariableNode) or not isinstance(rhs, LiteralNode):
            return None, None
        if not isinstance(rhs.value, int):
            return None, None
        if expr.operator == Operator.Inequality:
            return None, None
        return lhs.var_name, rhs.value

    def _inline_exit_if(
        self,
        block: BlockNode,
        exit_var: str,
        value: int,
        injected: List[StatementNode],
    ) -> bool:
        inserted = False
        i = 0
        pending_value: Optional[int] = None
        while i < len(block.statements):
            stmt = block.statements[i]
            if isinstance(stmt, (WhileNode, DoWhileNode, SwitchNode)):
                i += 1
                continue
            if isinstance(stmt, IfNode):
                if stmt.then_block:
                    inserted |= self._inline_exit_if(
                        stmt.then_block, exit_var, value, injected
                    )
                i += 1
                continue
            if isinstance(stmt, IfElseNode):
                if stmt.then_block:
                    inserted |= self._inline_exit_if(
                        stmt.then_block, exit_var, value, injected
                    )
                if stmt.else_block:
                    inserted |= self._inline_exit_if(
                        stmt.else_block, exit_var, value, injected
                    )
                i += 1
                continue
            if isinstance(stmt, AssignmentNode) and stmt.target == exit_var:
                literal = self._literal_int(stmt.value)
                if literal is not None:
                    pending_value = literal
                i += 1
                continue
            if isinstance(stmt, BreakNode) and pending_value == value:
                block.statements[i:i] = list(injected)
                inserted = True
                if i > 0:
                    prev_stmt = block.statements[i - 1]
                    if (
                        isinstance(prev_stmt, AssignmentNode)
                        and prev_stmt.target == exit_var
                    ):
                        block.statements.pop(i - 1)
                        i -= 1
                pending_value = None
            i += 1
        return inserted

    def _inline_exit_if_simple(
        self,
        block: BlockNode,
        exit_var: str,
        value: int,
        injected: List[StatementNode],
    ) -> bool:
        inserted = False
        i = 0
        while i < len(block.statements):
            stmt = block.statements[i]
            if isinstance(stmt, (WhileNode, DoWhileNode, SwitchNode)):
                i += 1
                continue
            if isinstance(stmt, IfNode):
                if stmt.then_block:
                    inserted |= self._inline_exit_if_simple(
                        stmt.then_block, exit_var, value, injected
                    )
                i += 1
                continue
            if isinstance(stmt, IfElseNode):
                if stmt.then_block:
                    inserted |= self._inline_exit_if_simple(
                        stmt.then_block, exit_var, value, injected
                    )
                if stmt.else_block:
                    inserted |= self._inline_exit_if_simple(
                        stmt.else_block, exit_var, value, injected
                    )
                i += 1
                continue
            if isinstance(stmt, AssignmentNode) and stmt.target == exit_var:
                literal = self._literal_int(stmt.value)
                next_stmt = (
                    block.statements[i + 1]
                    if i + 1 < len(block.statements)
                    else None
                )
                if literal == value and isinstance(next_stmt, BreakNode):
                    block.statements[i:i] = list(injected)
                    block.statements.pop(i + len(injected))
                    inserted = True
                    i += len(injected)
                    continue
            i += 1
        return inserted

    def _strip_trailing_break(
        self, statements: List[StatementNode]
    ) -> List[StatementNode]:
        if statements and isinstance(statements[-1], BreakNode):
            return list(statements[:-1])
        return list(statements)

    def _insert_before_break(
        self, site: _ExitSite, statements: List[StatementNode]
    ) -> None:
        if not statements:
            return
        site.block.statements[site.break_index:site.break_index] = list(statements)

    def _literal_int(self, expr: Optional[ExpressionNode]) -> Optional[int]:
        if isinstance(expr, LiteralNode) and isinstance(expr.value, int):
            return expr.value
        return None
