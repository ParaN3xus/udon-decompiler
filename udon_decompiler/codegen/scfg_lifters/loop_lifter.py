from __future__ import annotations

from typing import List, Optional

from udon_decompiler.analysis.expression_builder import Operator
from udon_decompiler.codegen.ast_nodes import (
    AssignmentNode,
    BlockNode,
    DoWhileNode,
    ExpressionNode,
    IfElseNode,
    IfNode,
    LiteralNode,
    OperatorNode,
    StatementNode,
    VariableNode,
    WhileNode,
)


class LoopLifter:
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
                if self._try_lift_loop(block, i, stmt):
                    continue
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
            i += 1

    def _try_lift_loop(self, parent: BlockNode, index: int, loop: WhileNode) -> bool:
        if not loop.body or not loop.body.statements:
            return False

        loop_cont = self._get_loop_cont_name(loop.condition)
        if loop_cont is None:
            return False

        body_stmts = loop.body.statements
        if len(body_stmts) < 2:
            return False

        last = body_stmts[-1]
        if not isinstance(last, AssignmentNode):
            return False
        if last.target != loop_cont:
            return False

        backedge_var, continue_value, is_equality = self._parse_loop_cont_assignment(
            last
        )
        if backedge_var is None or continue_value is None:
            return False

        # condition check is at the end -> do-while when
        if isinstance(body_stmts[-2], IfElseNode):
            if self._try_lift_do_while(
                parent,
                index,
                loop,
                loop_cont,
                backedge_var,
                continue_value,
                is_equality,
                body_stmts[-2],
            ):
                return True

        # while
        if isinstance(body_stmts[0], IfElseNode):
            return self._try_lift_while(
                parent,
                index,
                loop,
                loop_cont,
                backedge_var,
                continue_value,
                is_equality,
                body_stmts[0],
            )

        return self._try_remove_loop_cont(
            parent, index, loop, loop_cont, backedge_var, continue_value, is_equality
        )

    def _try_remove_loop_cont(
        self,
        parent: BlockNode,
        index: int,
        loop: WhileNode,
        loop_cont: str,
        backedge_var: str,
        continue_value: int,
        is_equality: bool,
    ) -> bool:
        # Rewrite loop-cont update into a break check.
        if not loop.body or not loop.body.statements:
            return False
        body = loop.body
        last = body.statements[-1]
        if not isinstance(last, AssignmentNode) or last.target != loop_cont:
            return False
        if continue_value not in (0, 1):
            return False

        # Replace loop_cont assignment with "if (backedge != continue_value) break;".
        if is_equality:
            op = Operator.Inequality
        else:
            op = Operator.Equality
        cond = OperatorNode(
            operator=op,
            receiver=None,
            operands=[
                VariableNode(var_name=backedge_var, var_type="System.Int32"),
                LiteralNode(value=continue_value, literal_type="System.Int32"),
            ],
        )
        then_block = BlockNode(statements=[self._break_statement()])
        body.statements[-1] = IfNode(condition=cond, then_block=then_block)

        loop.condition = LiteralNode(value=True, literal_type="System.Boolean")
        self._drop_loop_cont_init(parent, index, loop_cont)
        return True

    def _break_statement(self) -> StatementNode:
        from udon_decompiler.codegen.ast_nodes import BreakNode

        return BreakNode()

    def _try_lift_while(
        self,
        parent: BlockNode,
        index: int,
        loop: WhileNode,
        loop_cont: str,
        backedge_var: str,
        continue_value: int,
        is_equality: bool,
        cond_if: IfElseNode,
    ) -> bool:
        if loop.body is None:
            return False
        body_stmts = loop.body.statements
        if len(body_stmts) != 2:
            return False

        cond, loop_body = self._extract_loop_condition(
            cond_if, backedge_var, continue_value, is_equality
        )
        if cond is None or loop_body is None:
            return False

        lifted_loop = WhileNode(condition=cond, body=loop_body)
        parent.statements[index] = lifted_loop
        self._drop_loop_cont_init(parent, index, loop_cont)
        return True

    def _try_lift_do_while(
        self,
        parent: BlockNode,
        index: int,
        loop: WhileNode,
        loop_cont: str,
        backedge_var: str,
        continue_value: int,
        is_equality: bool,
        cond_if: IfElseNode,
    ) -> bool:
        if loop.body is None:
            return False
        body_stmts = loop.body.statements
        if len(body_stmts) < 2:
            return False

        cond, _ = self._extract_loop_condition(
            cond_if, backedge_var, continue_value, is_equality
        )
        if cond is None:
            return False

        body_without_tail = body_stmts[:-2]
        if not self._blocks_are_empty(cond_if.then_block, cond_if.else_block):
            return False

        lifted_loop = DoWhileNode(
            condition=cond, body=BlockNode(statements=list(body_without_tail))
        )
        parent.statements[index] = lifted_loop
        self._drop_loop_cont_init(parent, index, loop_cont)
        return True

    def _parse_loop_cont_assignment(
        self, assign: AssignmentNode
    ) -> tuple[Optional[str], Optional[int], bool]:
        value = assign.value
        if not isinstance(value, OperatorNode):
            return None, None, True
        if value.operator not in (Operator.Equality, Operator.Inequality):
            return None, None, True
        if len(value.operands) != 2:
            return None, None, True
        lhs, rhs = value.operands
        if not isinstance(lhs, VariableNode) or not isinstance(rhs, LiteralNode):
            return None, None, True
        if not isinstance(rhs.value, int):
            return None, None, True
        return lhs.var_name, rhs.value, value.operator == Operator.Equality

    def _extract_loop_condition(
        self,
        cond_if: IfElseNode,
        backedge_var: str,
        continue_value: int,
        is_equality: bool,
    ) -> tuple[Optional[ExpressionNode], Optional[BlockNode]]:
        then_val = self._extract_backedge_value(cond_if.then_block, backedge_var)
        else_val = self._extract_backedge_value(cond_if.else_block, backedge_var)
        if then_val is None or else_val is None:
            return None, None

        cond = cond_if.condition
        if cond is None:
            return None, None

        continue_on_then = then_val == continue_value
        continue_on_else = else_val == continue_value
        if continue_on_then == continue_on_else:
            return None, None

        if continue_on_then:
            loop_body = self._strip_backedge_assign(cond_if.then_block, backedge_var)
            return (cond if is_equality else self._invert_condition(cond), loop_body)

        loop_body = self._strip_backedge_assign(cond_if.else_block, backedge_var)
        return (self._invert_condition(cond) if is_equality else cond, loop_body)

    def _extract_backedge_value(
        self, block: Optional[BlockNode], backedge_var: str
    ) -> Optional[int]:
        if block is None or not block.statements:
            return None
        last = block.statements[-1]
        if not isinstance(last, AssignmentNode):
            return None
        if last.target != backedge_var:
            return None
        if not isinstance(last.value, LiteralNode):
            return None
        if not isinstance(last.value.value, int):
            return None
        return last.value.value

    def _strip_backedge_assign(
        self, block: Optional[BlockNode], backedge_var: str
    ) -> Optional[BlockNode]:
        if block is None:
            return None
        if not block.statements:
            return BlockNode()
        if (
            isinstance(block.statements[-1], AssignmentNode)
            and block.statements[-1].target == backedge_var
        ):
            return BlockNode(statements=list(block.statements[:-1]))
        return block

    def _blocks_are_empty(
        self, then_block: Optional[BlockNode], else_block: Optional[BlockNode]
    ) -> bool:
        for block in (then_block, else_block):
            if not block:
                continue
            if not block.statements:
                continue
            if len(block.statements) > 1:
                return False
            stmt = block.statements[0]
            if not isinstance(stmt, AssignmentNode):
                return False
        return True

    def _get_loop_cont_name(self, cond: Optional[ExpressionNode]) -> Optional[str]:
        if isinstance(cond, VariableNode):
            return cond.var_name
        return None

    def _invert_condition(self, condition: ExpressionNode) -> ExpressionNode:
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

    def _drop_loop_cont_init(
        self, parent: BlockNode, index: int, loop_cont: str
    ) -> None:
        if index == 0:
            return
        prev = parent.statements[index - 1]
        if isinstance(prev, AssignmentNode) and prev.target == loop_cont:
            parent.statements.pop(index - 1)
