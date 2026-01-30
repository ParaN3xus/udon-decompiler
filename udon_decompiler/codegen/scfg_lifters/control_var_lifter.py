from __future__ import annotations

from typing import List, Optional, Set

from udon_decompiler.analysis.expression_builder import Operator
from udon_decompiler.codegen.ast_nodes import (
    AssignmentNode,
    BlockNode,
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
    DoWhileNode,
    BreakNode,
)


class ControlVarLifter:
    def lift(self, statements: List[StatementNode]) -> List[StatementNode]:
        block = BlockNode(statements=statements)
        self._lift_block(block)
        return block.statements

    def _lift_block(self, block: BlockNode) -> None:
        i = 0
        while i < len(block.statements):
            stmt = block.statements[i]
            if isinstance(stmt, IfNode):
                if stmt.then_block:
                    self._lift_block(stmt.then_block)
            elif isinstance(stmt, IfElseNode):
                if stmt.then_block:
                    self._lift_block(stmt.then_block)
                if stmt.else_block:
                    self._lift_block(stmt.else_block)
            elif isinstance(stmt, SwitchNode):
                self._lift_block_in_switch(stmt)
            elif isinstance(stmt, WhileNode):
                if stmt.body:
                    self._lift_block(stmt.body)
            elif isinstance(stmt, DoWhileNode):
                if stmt.body:
                    self._lift_block(stmt.body)
            i += 1

        self._try_inline_control_var_default(block)

    def _lift_block_in_switch(self, switch: SwitchNode) -> None:
        for case in switch.cases:
            if case.body:
                self._lift_block(case.body)
        if switch.default_case and switch.default_case.body:
            self._lift_block(switch.default_case.body)

    def _try_inline_control_var_default(self, block: BlockNode) -> None:
        for idx, stmt in enumerate(list(block.statements)):
            if isinstance(stmt, IfNode):
                cond = stmt.condition
                then_block = stmt.then_block
                else_block = None
            elif isinstance(stmt, IfElseNode):
                cond = stmt.condition
                then_block = stmt.then_block
                else_block = stmt.else_block
            else:
                continue
            if cond is None:
                continue
            control_var, values = self._parse_control_var_condition(cond)
            if control_var is None or not values:
                continue
            if else_block and not self._block_is_effectively_empty(else_block):
                continue
            if not then_block or not then_block.statements:
                continue
            injected = list(then_block.statements)
            if not self._rewrite_control_var_assignments(
                block, control_var, values, injected
            ):
                continue
            block.statements.pop(idx)
            return

    def _parse_control_var_condition(
        self, expr: ExpressionNode
    ) -> tuple[Optional[str], Set[int]]:
        values: Set[int] = set()

        def collect(node: ExpressionNode) -> Optional[str]:
            if isinstance(node, OperatorNode) and node.operator == Operator.LogicalOr:
                if not node.operands:
                    return None
                var_name: Optional[str] = None
                for operand in node.operands:
                    sub = collect(operand)
                    if sub is None:
                        return None
                    if var_name is None:
                        var_name = sub
                    elif var_name != sub:
                        return None
                return var_name
            if isinstance(node, OperatorNode) and node.operator == Operator.Equality:
                if len(node.operands) != 2:
                    return None
                lhs, rhs = node.operands
                if isinstance(lhs, LiteralNode) and isinstance(rhs, VariableNode):
                    lhs, rhs = rhs, lhs
                if isinstance(lhs, VariableNode) and isinstance(rhs, LiteralNode):
                    if isinstance(rhs.value, int):
                        values.add(rhs.value)
                        return lhs.var_name
            return None

        var = collect(expr)
        return var, values

    def _block_is_effectively_empty(self, block: BlockNode) -> bool:
        if not block.statements:
            return True
        for stmt in block.statements:
            if isinstance(stmt, ExpressionStatementNode) and stmt.expression is None:
                continue
            return False
        return True

    def _rewrite_control_var_assignments(
        self,
        block: BlockNode,
        control_var: str,
        values: Set[int],
        injected: List[StatementNode],
    ) -> bool:
        changed = False
        unsupported = False
        i = 0
        while i < len(block.statements):
            stmt = block.statements[i]
            if isinstance(stmt, SwitchNode):
                local_changed, local_unsupported = self._rewrite_switch(
                    stmt, control_var, values, injected
                )
                changed |= local_changed
                unsupported |= local_unsupported
                i += 1
                continue
            if isinstance(stmt, IfElseNode):
                if stmt.then_block:
                    changed |= self._rewrite_control_var_assignments(
                        stmt.then_block, control_var, values, injected
                    )
                if stmt.else_block:
                    changed |= self._rewrite_control_var_assignments(
                        stmt.else_block, control_var, values, injected
                    )
                i += 1
                continue
            if isinstance(stmt, IfNode):
                if stmt.then_block:
                    changed |= self._rewrite_control_var_assignments(
                        stmt.then_block, control_var, values, injected
                    )
                i += 1
                continue
            if isinstance(stmt, AssignmentNode) and stmt.target == control_var:
                literal = self._literal_int(stmt.value)
                if literal is None:
                    unsupported = True
                    i += 1
                    continue
                if literal in values:
                    block.statements[i:i + 1] = list(injected)
                    changed = True
                    i += len(injected)
                    continue
                block.statements.pop(i)
                changed = True
                continue
            i += 1
        return changed and not unsupported

    def _rewrite_switch(
        self,
        switch: SwitchNode,
        control_var: str,
        values: Set[int],
        injected: List[StatementNode],
    ) -> tuple[bool, bool]:
        changed = False
        unsupported = False
        removable_cases: List[SwitchCaseNode] = []
        control_only_default = self._case_is_control_only(
            switch.default_case, control_var
        )
        default_ok = switch.default_case is None or control_only_default
        for case in switch.cases:
            if not case.body:
                continue
            literal, has_other, literal_ok = self._case_control_literal(
                case, control_var
            )
            if not literal_ok:
                unsupported = True
                continue
            if (
                literal is not None
                and literal in values
                and not has_other
                and default_ok
            ):
                removable_cases.append(case)
                changed = True
                continue
            if literal is not None and literal in values:
                changed |= self._replace_control_var_assign(
                    case.body, control_var, injected
                )
            else:
                changed |= self._strip_control_var_assign(case.body, control_var)
        if switch.default_case and switch.default_case.body:
            changed |= self._strip_control_var_assign(
                switch.default_case.body, control_var
            )
        if removable_cases and default_ok:
            switch.cases = [c for c in switch.cases if c not in removable_cases]
            if switch.default_case is None:
                switch.default_case = SwitchCaseNode(
                    values=[], body=BlockNode(statements=[]), is_default=True
                )
            if switch.default_case.body is None:
                switch.default_case.body = BlockNode(statements=[])
            switch.default_case.body.statements = list(injected)
            switch.default_case.body.statements.append(BreakNode())
            changed = True
        return changed, unsupported

    def _case_control_literal(
        self, case: SwitchCaseNode, control_var: str
    ) -> tuple[Optional[int], bool, bool]:
        if case.body is None or not case.body.statements:
            return None, False, True
        literal: Optional[int] = None
        has_other = False
        for stmt in case.body.statements:
            if isinstance(stmt, BreakNode):
                continue
            if isinstance(stmt, AssignmentNode) and stmt.target == control_var:
                if literal is not None:
                    return None, True, False
                literal = self._literal_int(stmt.value)
                if literal is None:
                    return None, True, False
                continue
            has_other = True
        return literal, has_other, True

    def _case_is_control_only(
        self, case: Optional[SwitchCaseNode], control_var: str
    ) -> bool:
        if case is None or case.body is None or not case.body.statements:
            return True
        if all(isinstance(stmt, BreakNode) for stmt in case.body.statements):
            return True
        literal, has_other, ok = self._case_control_literal(case, control_var)
        return ok and not has_other and literal is not None

    def _replace_control_var_assign(
        self, block: BlockNode, control_var: str, injected: List[StatementNode]
    ) -> bool:
        changed = False
        i = 0
        while i < len(block.statements):
            stmt = block.statements[i]
            if isinstance(stmt, AssignmentNode) and stmt.target == control_var:
                block.statements[i:i + 1] = list(injected)
                changed = True
                i += len(injected)
                continue
            if isinstance(stmt, IfNode):
                if stmt.then_block:
                    changed |= self._replace_control_var_assign(
                        stmt.then_block, control_var, injected
                    )
            if isinstance(stmt, IfElseNode):
                if stmt.then_block:
                    changed |= self._replace_control_var_assign(
                        stmt.then_block, control_var, injected
                    )
                if stmt.else_block:
                    changed |= self._replace_control_var_assign(
                        stmt.else_block, control_var, injected
                    )
            i += 1
        return changed

    def _strip_control_var_assign(self, block: BlockNode, control_var: str) -> bool:
        changed = False
        i = 0
        while i < len(block.statements):
            stmt = block.statements[i]
            if isinstance(stmt, AssignmentNode) and stmt.target == control_var:
                block.statements.pop(i)
                changed = True
                continue
            if isinstance(stmt, IfNode):
                if stmt.then_block:
                    changed |= self._strip_control_var_assign(
                        stmt.then_block, control_var
                    )
            if isinstance(stmt, IfElseNode):
                if stmt.then_block:
                    changed |= self._strip_control_var_assign(
                        stmt.then_block, control_var
                    )
                if stmt.else_block:
                    changed |= self._strip_control_var_assign(
                        stmt.else_block, control_var
                    )
            i += 1
        return changed

    def _literal_int(self, expr: Optional[ExpressionNode]) -> Optional[int]:
        if isinstance(expr, LiteralNode) and isinstance(expr.value, int):
            return expr.value
        return None
