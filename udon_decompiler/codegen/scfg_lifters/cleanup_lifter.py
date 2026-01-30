from __future__ import annotations

from typing import List, Set

from udon_decompiler.codegen.ast_nodes import (
    AssignmentNode,
    BlockNode,
    DoWhileNode,
    ExpressionNode,
    ExpressionStatementNode,
    IfElseNode,
    IfNode,
    StatementNode,
    SwitchCaseNode,
    SwitchNode,
    VariableDeclNode,
    VariableNode,
    WhileNode,
)


class CleanupLifter:
    def lift(self, statements: List[StatementNode]) -> List[StatementNode]:
        block = BlockNode(statements=statements)
        self._cleanup_block(block)
        reads = self.collect_read_variable_names(block)
        self._strip_unused_assignments(block, reads)
        self._strip_unused_scfg_decls(block, reads)
        return block.statements

    def _cleanup_block(self, block: BlockNode) -> None:
        i = 0
        while i < len(block.statements):
            stmt = block.statements[i]
            if isinstance(stmt, ExpressionStatementNode) and stmt.expression is None:
                block.statements.pop(i)
                continue
            if isinstance(stmt, IfNode):
                if stmt.then_block:
                    self._cleanup_block(stmt.then_block)
                if stmt.then_block and not stmt.then_block.statements:
                    block.statements.pop(i)
                    continue
            elif isinstance(stmt, IfElseNode):
                if stmt.then_block:
                    self._cleanup_block(stmt.then_block)
                if stmt.else_block:
                    self._cleanup_block(stmt.else_block)
                if (
                    stmt.then_block
                    and not stmt.then_block.statements
                    and stmt.else_block
                ):
                    # Flip to avoid empty then.
                    stmt.condition = self._invert_condition(stmt.condition)
                    stmt.then_block, stmt.else_block = stmt.else_block, stmt.then_block
                if (
                    stmt.then_block
                    and not stmt.then_block.statements
                    and (stmt.else_block is None or not stmt.else_block.statements)
                ):
                    block.statements.pop(i)
                    continue
            elif isinstance(stmt, WhileNode):
                if stmt.body:
                    self._cleanup_block(stmt.body)
            elif isinstance(stmt, DoWhileNode):
                if stmt.body:
                    self._cleanup_block(stmt.body)
            elif isinstance(stmt, SwitchNode):
                self._cleanup_switch(stmt)
            i += 1

    def _cleanup_switch(self, switch: SwitchNode) -> None:
        cases = []
        for case in switch.cases:
            if case.body:
                self._cleanup_block(case.body)
            if case.body is None or not case.body.statements:
                continue
            cases.append(case)
        switch.cases = cases

    def _strip_unused_assignments(self, block: BlockNode, reads: Set[str]) -> None:
        i = 0
        while i < len(block.statements):
            stmt = block.statements[i]
            if isinstance(stmt, AssignmentNode) and stmt.target.startswith("__scfg_"):
                if stmt.target not in reads:
                    block.statements.pop(i)
                    continue
            if isinstance(stmt, IfNode):
                if stmt.then_block:
                    self._strip_unused_assignments(stmt.then_block, reads)
            elif isinstance(stmt, IfElseNode):
                if stmt.then_block:
                    self._strip_unused_assignments(stmt.then_block, reads)
                if stmt.else_block:
                    self._strip_unused_assignments(stmt.else_block, reads)
            elif isinstance(stmt, WhileNode):
                if stmt.body:
                    self._strip_unused_assignments(stmt.body, reads)
            elif isinstance(stmt, DoWhileNode):
                if stmt.body:
                    self._strip_unused_assignments(stmt.body, reads)
            elif isinstance(stmt, SwitchNode):
                for case in stmt.cases:
                    if case.body:
                        self._strip_unused_assignments(case.body, reads)
                if stmt.default_case and stmt.default_case.body:
                    self._strip_unused_assignments(stmt.default_case.body, reads)
            i += 1

    def _strip_unused_scfg_decls(self, block: BlockNode, reads: Set[str]) -> None:
        i = 0
        while i < len(block.statements):
            stmt = block.statements[i]
            if isinstance(stmt, VariableDeclNode) and stmt.var_name.startswith(
                "__scfg_"
            ):
                if stmt.var_name not in reads:
                    block.statements.pop(i)
                    continue
            if isinstance(stmt, IfNode):
                if stmt.then_block:
                    self._strip_unused_scfg_decls(stmt.then_block, reads)
            elif isinstance(stmt, IfElseNode):
                if stmt.then_block:
                    self._strip_unused_scfg_decls(stmt.then_block, reads)
                if stmt.else_block:
                    self._strip_unused_scfg_decls(stmt.else_block, reads)
            elif isinstance(stmt, WhileNode):
                if stmt.body:
                    self._strip_unused_scfg_decls(stmt.body, reads)
            elif isinstance(stmt, DoWhileNode):
                if stmt.body:
                    self._strip_unused_scfg_decls(stmt.body, reads)
            elif isinstance(stmt, SwitchNode):
                for case in stmt.cases:
                    if case.body:
                        self._strip_unused_scfg_decls(case.body, reads)
                if stmt.default_case and stmt.default_case.body:
                    self._strip_unused_scfg_decls(stmt.default_case.body, reads)
            i += 1

    @staticmethod
    def collect_referenced_variable_names(statements: List[StatementNode]) -> Set[str]:
        referenced: Set[str] = set()
        CleanupLifter._collect_from_block(BlockNode(statements=statements), referenced)
        return referenced

    @staticmethod
    def collect_read_variable_names(block: BlockNode) -> Set[str]:
        referenced: Set[str] = set()
        CleanupLifter._collect_reads_from_block(block, referenced)
        return referenced

    @staticmethod
    def _collect_reads_from_block(block: BlockNode, referenced: Set[str]) -> None:
        for stmt in block.statements:
            CleanupLifter._collect_reads_from_statement(stmt, referenced)

    @staticmethod
    def _collect_reads_from_statement(
        stmt: StatementNode, referenced: Set[str]
    ) -> None:
        if isinstance(stmt, AssignmentNode):
            if stmt.value:
                CleanupLifter._collect_from_expression(stmt.value, referenced)
            return
        if isinstance(stmt, ExpressionStatementNode):
            if stmt.expression:
                CleanupLifter._collect_from_expression(stmt.expression, referenced)
            return
        if isinstance(stmt, IfNode):
            if stmt.condition:
                CleanupLifter._collect_from_expression(stmt.condition, referenced)
            if stmt.then_block:
                CleanupLifter._collect_reads_from_block(stmt.then_block, referenced)
            return
        if isinstance(stmt, IfElseNode):
            if stmt.condition:
                CleanupLifter._collect_from_expression(stmt.condition, referenced)
            if stmt.then_block:
                CleanupLifter._collect_reads_from_block(stmt.then_block, referenced)
            if stmt.else_block:
                CleanupLifter._collect_reads_from_block(stmt.else_block, referenced)
            return
        if isinstance(stmt, WhileNode):
            if stmt.condition:
                CleanupLifter._collect_from_expression(stmt.condition, referenced)
            if stmt.body:
                CleanupLifter._collect_reads_from_block(stmt.body, referenced)
            return
        if isinstance(stmt, DoWhileNode):
            if stmt.condition:
                CleanupLifter._collect_from_expression(stmt.condition, referenced)
            if stmt.body:
                CleanupLifter._collect_reads_from_block(stmt.body, referenced)
            return
        if isinstance(stmt, SwitchNode):
            if stmt.expression:
                CleanupLifter._collect_from_expression(stmt.expression, referenced)
            for case in stmt.cases:
                for value in case.values:
                    CleanupLifter._collect_from_expression(value, referenced)
                if case.body:
                    CleanupLifter._collect_reads_from_block(case.body, referenced)
            if stmt.default_case and stmt.default_case.body:
                CleanupLifter._collect_reads_from_block(
                    stmt.default_case.body, referenced
                )
            return

    @staticmethod
    def _collect_from_block(block: BlockNode, referenced: Set[str]) -> None:
        for stmt in block.statements:
            CleanupLifter._collect_from_statement(stmt, referenced)

    @staticmethod
    def _collect_from_statement(stmt: StatementNode, referenced: Set[str]) -> None:
        if isinstance(stmt, AssignmentNode):
            if stmt.target:
                referenced.add(stmt.target)
            if stmt.value:
                CleanupLifter._collect_from_expression(stmt.value, referenced)
            return
        if isinstance(stmt, ExpressionStatementNode):
            if stmt.expression:
                CleanupLifter._collect_from_expression(stmt.expression, referenced)
            return
        if isinstance(stmt, IfNode):
            if stmt.condition:
                CleanupLifter._collect_from_expression(stmt.condition, referenced)
            if stmt.then_block:
                CleanupLifter._collect_from_block(stmt.then_block, referenced)
            return
        if isinstance(stmt, IfElseNode):
            if stmt.condition:
                CleanupLifter._collect_from_expression(stmt.condition, referenced)
            if stmt.then_block:
                CleanupLifter._collect_from_block(stmt.then_block, referenced)
            if stmt.else_block:
                CleanupLifter._collect_from_block(stmt.else_block, referenced)
            return
        if isinstance(stmt, WhileNode):
            if stmt.condition:
                CleanupLifter._collect_from_expression(stmt.condition, referenced)
            if stmt.body:
                CleanupLifter._collect_from_block(stmt.body, referenced)
            return
        if isinstance(stmt, DoWhileNode):
            if stmt.condition:
                CleanupLifter._collect_from_expression(stmt.condition, referenced)
            if stmt.body:
                CleanupLifter._collect_from_block(stmt.body, referenced)
            return
        if isinstance(stmt, SwitchNode):
            if stmt.expression:
                CleanupLifter._collect_from_expression(stmt.expression, referenced)
            for case in stmt.cases:
                for value in case.values:
                    CleanupLifter._collect_from_expression(value, referenced)
                if case.body:
                    CleanupLifter._collect_from_block(case.body, referenced)
            if stmt.default_case and stmt.default_case.body:
                CleanupLifter._collect_from_block(stmt.default_case.body, referenced)
            return

    @staticmethod
    def _collect_from_expression(expr: ExpressionNode, referenced: Set[str]) -> None:
        if isinstance(expr, VariableNode):
            if expr.var_name:
                referenced.add(expr.var_name)
            return
        for child in getattr(expr, "operands", []):
            CleanupLifter._collect_from_expression(child, referenced)
        for child in getattr(expr, "arguments", []):
            if isinstance(child, tuple) and child[1]:
                CleanupLifter._collect_from_expression(child[1], referenced)
        for attr in ("receiver", "this", "target", "value"):
            child = getattr(expr, attr, None)
            if child is not None:
                CleanupLifter._collect_from_expression(child, referenced)

    def _invert_condition(
        self, condition: ExpressionNode | None
    ) -> ExpressionNode | None:
        if condition is None:
            return None
        # Keep it simple: reuse operator negation via AST nodes.
        from udon_decompiler.analysis.expression_builder import Operator
        from udon_decompiler.codegen.ast_nodes import OperatorNode

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
