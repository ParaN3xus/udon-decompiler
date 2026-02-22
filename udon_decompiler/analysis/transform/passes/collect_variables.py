from __future__ import annotations

from typing import Dict

from udon_decompiler.analysis.ir.nodes import (
    IRAssignmentStatement,
    IRBlock,
    IRBlockContainer,
    IRConstructorCallExpression,
    IRExpression,
    IRExpressionStatement,
    IRExternalCallExpression,
    IRFunction,
    IRHighLevelSwitch,
    IRIf,
    IRInternalCallExpression,
    IRLiteralExpression,
    IROperatorCallExpression,
    IRPropertyAccessExpression,
    IRStatement,
    IRSwitch,
    IRVariableDeclarationStatement,
    IRVariableExpression,
)
from udon_decompiler.analysis.transform.pass_base import (
    IILTransform,
    ILTransformContext,
)
from udon_decompiler.analysis.variable_identifier import Variable


class CollectVariables(IILTransform):
    """Collect referenced variables into ``function.variable_declarations``."""

    def run(self, function: IRFunction, context: ILTransformContext) -> None:
        _ = context

        seen_blocks: set[int] = set()
        collected: Dict[int, Variable] = {}

        self._collect_from_container(function.body, seen_blocks, collected)

        declarations = [
            IRVariableDeclarationStatement(variable=var, init_value=None)
            for _, var in sorted(collected.items(), key=lambda item: item[0])
        ]
        function.variable_declarations = declarations

    def _collect_from_container(
        self,
        container: IRBlockContainer,
        seen_blocks: set[int],
        collected: Dict[int, Variable],
    ) -> None:
        for block in container.blocks:
            self._collect_from_block(block, seen_blocks, collected)

    def _collect_from_block(
        self,
        block: IRBlock,
        seen_blocks: set[int],
        collected: Dict[int, Variable],
    ) -> None:
        block_id = id(block)
        if block_id in seen_blocks:
            return
        seen_blocks.add(block_id)

        for statement in block.statements:
            self._collect_from_statement(statement, seen_blocks, collected)

    def _collect_from_statement(
        self,
        statement: IRStatement,
        seen_blocks: set[int],
        collected: Dict[int, Variable],
    ) -> None:
        if isinstance(statement, IRAssignmentStatement):
            self._collect_variable(statement.target, collected)
            self._collect_from_expression(statement.value, collected)
            return

        if isinstance(statement, IRExpressionStatement):
            self._collect_from_expression(statement.expression, collected)
            return

        if isinstance(statement, IRIf):
            self._collect_from_expression(statement.condition, collected)
            self._collect_from_statement(
                statement.true_statement,
                seen_blocks,
                collected,
            )
            if statement.false_statement is not None:
                self._collect_from_statement(
                    statement.false_statement,
                    seen_blocks,
                    collected,
                )
            return

        if isinstance(statement, IRBlock):
            self._collect_from_block(statement, seen_blocks, collected)
            return

        if isinstance(statement, IRBlockContainer):
            self._collect_from_container(statement, seen_blocks, collected)
            return

        if isinstance(statement, IRSwitch):
            self._collect_from_expression(statement.index_expression, collected)
            return

        if isinstance(statement, IRHighLevelSwitch):
            self._collect_from_expression(statement.index_expression, collected)
            for section in statement.sections:
                self._collect_from_block(section.body, seen_blocks, collected)
            return

    def _collect_from_expression(
        self,
        expression: IRExpression,
        collected: Dict[int, Variable],
    ) -> None:
        if isinstance(expression, IRVariableExpression):
            self._collect_variable(expression.variable, collected)
            return

        if isinstance(
            expression,
            (
                IRExternalCallExpression,
                IRPropertyAccessExpression,
                IRConstructorCallExpression,
                IROperatorCallExpression,
            ),
        ):
            for argument in expression.arguments:
                self._collect_from_expression(argument, collected)
            return

        if isinstance(expression, (IRLiteralExpression, IRInternalCallExpression)):
            return

    def _collect_variable(
        self,
        variable: Variable,
        collected: Dict[int, Variable],
    ) -> None:
        # Exclude pseudo receiver names from declaration emission.
        if variable.name == "this" or "." in variable.name:
            return

        if variable.address not in collected:
            collected[variable.address] = variable
