from __future__ import annotations

from typing import List, Optional

from udon_decompiler.analysis.expression_builder import Operator
from udon_decompiler.codegen.ast_nodes import (
    AssignmentNode,
    BlockNode,
    BreakNode,
    CallNode,
    ConstructionNode,
    DoWhileNode,
    ExpressionNode,
    ExpressionStatementNode,
    IfElseNode,
    IfNode,
    LiteralNode,
    OperatorNode,
    PropertyAccessNode,
    StatementNode,
    TypeNode,
    VariableNode,
    WhileNode,
)


class WhileTrueLifter:
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
                lifted = self._try_lift_while_true(stmt)
                if lifted is not None:
                    block.statements[i] = lifted
            elif isinstance(stmt, IfNode):
                if stmt.then_block:
                    self._lift_block(stmt.then_block)
            elif isinstance(stmt, IfElseNode):
                if stmt.then_block:
                    self._lift_block(stmt.then_block)
                if stmt.else_block:
                    self._lift_block(stmt.else_block)
            elif isinstance(stmt, DoWhileNode):
                if stmt.body:
                    self._lift_block(stmt.body)
            i += 1

    def _try_lift_while_true(self, loop: WhileNode) -> Optional[StatementNode]:
        if not self._is_literal_true(loop.condition):
            return None
        if not loop.body or not loop.body.statements:
            return None

        # Pattern A: while(true) { if (cond) { then } else { break; } rest... }
        first = loop.body.statements[0]
        if isinstance(first, IfElseNode) and first.then_block:
            if self._block_is_break_only(first.else_block):
                new_body = BlockNode(
                    statements=[
                        *first.then_block.statements,
                        *loop.body.statements[1:],
                    ]
                )
                return WhileNode(condition=first.condition, body=new_body)

        # Pattern A2: while(true) { tmp = cond; if (tmp) { then } else { break; } }
        # Then rest...
        if len(loop.body.statements) >= 2:
            assign_stmt = loop.body.statements[0]
            if_stmt = loop.body.statements[1]
            assignment = self._extract_temp_assignment(assign_stmt)
            if (
                assignment is not None
                and isinstance(if_stmt, IfElseNode)
                and if_stmt.then_block
                and isinstance(if_stmt.condition, VariableNode)
                and if_stmt.condition.var_name == assignment[0]
                and self._block_is_break_only(if_stmt.else_block)
            ):
                target, value_expr = assignment
                used_elsewhere = self._block_reads_var(
                    BlockNode(
                        statements=[
                            *if_stmt.then_block.statements,
                            *loop.body.statements[2:],
                        ]
                    ),
                    target,
                )
                if not used_elsewhere and value_expr is not None:
                    new_body = BlockNode(
                        statements=[
                            *if_stmt.then_block.statements,
                            *loop.body.statements[2:],
                        ]
                    )
                    return WhileNode(condition=value_expr, body=new_body)

        # Pattern B: while(true) { ...; if (cond) break; } => do { ... } while (!cond)
        last = loop.body.statements[-1]
        if isinstance(last, IfNode) and self._if_is_break_only(last):
            cond = self._invert_condition(last.condition)
            new_body = BlockNode(statements=loop.body.statements[:-1])
            return DoWhileNode(condition=cond, body=new_body)

        # Pattern C:
        # while(true) { ...; if (flag) break;
        # else { body; if (cond) break; } }
        last = loop.body.statements[-1]
        if isinstance(last, IfElseNode) and self._block_is_break_only(last.then_block):
            tail_cond, trimmed = self._strip_trailing_break_if(last.else_block)
            if tail_cond is not None and trimmed is not None:
                new_body = BlockNode(
                    statements=[
                        *loop.body.statements[:-1],
                        IfNode(condition=last.condition, then_block=last.then_block),
                        *trimmed.statements,
                    ]
                )
                return DoWhileNode(
                    condition=self._invert_condition(tail_cond), body=new_body
                )

        return None

    def _if_is_break_only(self, stmt: IfNode) -> bool:
        if stmt.then_block is None:
            return False
        return self._block_is_break_only(stmt.then_block)

    def _block_is_break_only(self, block: Optional[BlockNode]) -> bool:
        if block is None or not block.statements:
            return False
        for stmt in block.statements:
            if isinstance(stmt, BreakNode):
                continue
            if isinstance(stmt, ExpressionStatementNode) and stmt.expression is None:
                continue
            return False
        return True

    def _is_literal_true(self, expr: Optional[ExpressionNode]) -> bool:
        if isinstance(expr, LiteralNode) and isinstance(expr.value, bool):
            return expr.value is True
        return False

    def _invert_condition(self, condition: Optional[ExpressionNode]) -> ExpressionNode:
        if condition is None:
            return LiteralNode(literal_type="System.Boolean", value=True)
        if (
            isinstance(condition, OperatorNode)
            and condition.operator == Operator.UnaryNegation
            and condition.operands
        ):
            return condition.operands[0]
        return OperatorNode(
            operator=Operator.UnaryNegation,
            receiver=None,
            operands=[condition],
        )

    def _strip_trailing_break_if(
        self, block: Optional[BlockNode]
    ) -> tuple[Optional[ExpressionNode], Optional[BlockNode]]:
        if block is None or not block.statements:
            return None, None
        last = block.statements[-1]
        if isinstance(last, IfNode) and self._if_is_break_only(last):
            return last.condition, BlockNode(statements=block.statements[:-1])
        if isinstance(last, IfElseNode):
            if self._block_is_break_only(
                last.then_block
            ) and self._block_is_effectively_empty(last.else_block):
                return last.condition, BlockNode(statements=block.statements[:-1])
            if self._block_is_break_only(
                last.else_block
            ) and self._block_is_effectively_empty(last.then_block):
                return self._invert_condition(last.condition), BlockNode(
                    statements=block.statements[:-1]
                )
        return None, None

    def _extract_temp_assignment(
        self, stmt: StatementNode
    ) -> Optional[tuple[str, Optional[ExpressionNode]]]:
        if isinstance(stmt, AssignmentNode):
            return stmt.target, stmt.value
        if isinstance(stmt, ExpressionStatementNode) and isinstance(
            stmt.expression, OperatorNode
        ):
            expr = stmt.expression
            if expr.receiver is None or not isinstance(expr.receiver, VariableNode):
                return None
            value_expr = OperatorNode(
                operator=expr.operator,
                receiver=None,
                operands=list(expr.operands),
                emit_as_expression=True,
            )
            return expr.receiver.var_name, value_expr
        return None

    def _block_is_effectively_empty(self, block: Optional[BlockNode]) -> bool:
        if block is None or not block.statements:
            return True
        for stmt in block.statements:
            if isinstance(stmt, ExpressionStatementNode) and stmt.expression is None:
                continue
            return False
        return True

    def _block_reads_var(self, block: BlockNode, var_name: str) -> bool:
        for stmt in block.statements:
            if self._statement_reads_var(stmt, var_name):
                return True
        return False

    def _statement_reads_var(self, stmt: StatementNode, var_name: str) -> bool:
        if isinstance(stmt, AssignmentNode):
            return self._expr_reads_var(stmt.value, var_name)
        if isinstance(stmt, ExpressionStatementNode):
            return self._expr_reads_var(stmt.expression, var_name)
        if isinstance(stmt, IfNode):
            if self._expr_reads_var(stmt.condition, var_name):
                return True
            if stmt.then_block and self._block_reads_var(stmt.then_block, var_name):
                return True
            return False
        if isinstance(stmt, IfElseNode):
            if self._expr_reads_var(stmt.condition, var_name):
                return True
            if stmt.then_block and self._block_reads_var(stmt.then_block, var_name):
                return True
            if stmt.else_block and self._block_reads_var(stmt.else_block, var_name):
                return True
            return False
        if isinstance(stmt, WhileNode):
            if self._expr_reads_var(stmt.condition, var_name):
                return True
            if stmt.body and self._block_reads_var(stmt.body, var_name):
                return True
            return False
        if isinstance(stmt, DoWhileNode):
            if self._expr_reads_var(stmt.condition, var_name):
                return True
            if stmt.body and self._block_reads_var(stmt.body, var_name):
                return True
            return False
        return False

    def _expr_reads_var(self, expr: Optional[ExpressionNode], var_name: str) -> bool:
        if expr is None:
            return False
        if isinstance(expr, VariableNode):
            return expr.var_name == var_name
        if isinstance(expr, OperatorNode):
            if expr.receiver and self._expr_reads_var(expr.receiver, var_name):
                return True
            return any(self._expr_reads_var(op, var_name) for op in expr.operands)
        if isinstance(expr, CallNode):
            if expr.receiver and self._expr_reads_var(expr.receiver, var_name):
                return True
            for _, arg in expr.arguments:
                if self._expr_reads_var(arg, var_name):
                    return True
            return False
        if isinstance(expr, PropertyAccessNode):
            if expr.this and self._expr_reads_var(expr.this, var_name):
                return True
            if expr.target and self._expr_reads_var(expr.target, var_name):
                return True
            if expr.value and self._expr_reads_var(expr.value, var_name):
                return True
            return False
        if isinstance(expr, ConstructionNode):
            if expr.receiver and self._expr_reads_var(expr.receiver, var_name):
                return True
            return any(self._expr_reads_var(arg, var_name) for arg in expr.arguments)
        if isinstance(expr, TypeNode):
            return False
        return False
