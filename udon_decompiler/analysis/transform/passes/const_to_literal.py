from __future__ import annotations

from typing import Dict, Optional, Tuple

from udon_decompiler.analysis.ir.nodes import (
    IRAssignmentStatement,
    IRBlock,
    IRBlockContainer,
    IRConstructorCallExpression,
    IRExpression,
    IRExpressionStatement,
    IRExternalCallExpression,
    IRIf,
    IRLiteralExpression,
    IROperatorCallExpression,
    IRPropertyAccessExpression,
    IRStatement,
    IRSwitch,
    IRVariableExpression,
)
from udon_decompiler.analysis.transform.pass_base import (
    ITransform,
    TransformContext,
)
from udon_decompiler.models.program import SymbolInfo, UdonProgramData


class ConstToLiteral(ITransform):
    """Replace ``__const_*`` variable references with literal expressions."""

    def run(self, function, context: TransformContext) -> None:
        literal_cache: Dict[int, Tuple[object, Optional[str]]] = {}
        self._rewrite_container(
            container=function.body,
            program=context.program,
            literal_cache=literal_cache,
        )

    def _rewrite_container(
        self,
        container: IRBlockContainer,
        program: UdonProgramData,
        literal_cache: Dict[int, Tuple[object, Optional[str]]],
    ) -> None:
        for block in container.blocks:
            rewritten: list[IRStatement] = []
            for statement in block.statements:
                rewritten.append(
                    self._rewrite_statement(
                        statement=statement,
                        program=program,
                        literal_cache=literal_cache,
                    )
                )
            block.statements = rewritten

    def _rewrite_statement(
        self,
        statement: IRStatement,
        program: UdonProgramData,
        literal_cache: Dict[int, Tuple[object, Optional[str]]],
    ) -> IRStatement:
        if isinstance(statement, IRAssignmentStatement):
            statement.value = self._rewrite_expression(
                statement.value,
                program,
                literal_cache,
            )
            return statement

        if isinstance(statement, IRExpressionStatement):
            statement.expression = self._rewrite_expression(
                statement.expression,
                program,
                literal_cache,
            )
            return statement

        if isinstance(statement, IRIf):
            statement.condition = self._rewrite_expression(
                statement.condition,
                program,
                literal_cache,
            )
            statement.true_statement = self._rewrite_statement(
                statement.true_statement,
                program,
                literal_cache,
            )
            if statement.false_statement is not None:
                statement.false_statement = self._rewrite_statement(
                    statement.false_statement,
                    program,
                    literal_cache,
                )
            return statement

        if isinstance(statement, IRBlock):
            rewritten: list[IRStatement] = []
            for nested in statement.statements:
                rewritten.append(
                    self._rewrite_statement(
                        nested,
                        program,
                        literal_cache,
                    )
                )
            statement.statements = rewritten
            return statement

        if isinstance(statement, IRBlockContainer):
            self._rewrite_container(statement, program, literal_cache)
            return statement

        if isinstance(statement, IRSwitch):
            statement.index_expression = self._rewrite_expression(
                statement.index_expression,
                program,
                literal_cache,
            )
            return statement

        return statement

    def _rewrite_expression(
        self,
        expression: IRExpression,
        program: UdonProgramData,
        literal_cache: Dict[int, Tuple[object, Optional[str]]],
    ) -> IRExpression:
        if isinstance(expression, IRVariableExpression):
            literal = self._const_literal_for_variable(
                expression,
                program,
                literal_cache,
            )
            if literal is not None:
                return literal
            return expression

        if isinstance(expression, IROperatorCallExpression):
            expression.arguments = [
                self._rewrite_expression(arg, program, literal_cache)
                for arg in expression.arguments
            ]
            return expression

        if isinstance(expression, IRExternalCallExpression):
            expression.arguments = [
                self._rewrite_expression(arg, program, literal_cache)
                for arg in expression.arguments
            ]
            return expression

        if isinstance(expression, IRPropertyAccessExpression):
            expression.arguments = [
                self._rewrite_expression(arg, program, literal_cache)
                for arg in expression.arguments
            ]
            return expression

        if isinstance(expression, IRConstructorCallExpression):
            expression.arguments = [
                self._rewrite_expression(arg, program, literal_cache)
                for arg in expression.arguments
            ]
            return expression

        return expression

    def _const_literal_for_variable(
        self,
        expression: IRVariableExpression,
        program: UdonProgramData,
        literal_cache: Dict[int, Tuple[object, Optional[str]]],
    ) -> Optional[IRLiteralExpression]:
        variable = expression.variable
        if not variable.name.startswith(SymbolInfo.CONST_SYMBOL_PREFIX):
            return None

        if variable.address not in literal_cache:
            heap_entry = program.get_initial_heap_value(variable.address)
            if heap_entry is None or not heap_entry.value.is_serializable:
                return None
            literal_cache[variable.address] = (
                heap_entry.value.value,
                variable.type_hint or heap_entry.type,
            )

        value, type_hint = literal_cache[variable.address]
        return IRLiteralExpression(value=value, type_hint=type_hint)
