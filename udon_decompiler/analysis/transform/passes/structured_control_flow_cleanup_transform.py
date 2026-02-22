from __future__ import annotations

from typing import cast

from udon_decompiler.analysis.ir.nodes import (
    IRBlock,
    IRBlockContainer,
    IRFunction,
    IRHighLevelSwitch,
    IRIf,
    IRStatement,
)
from udon_decompiler.analysis.transform.pass_base import (
    IILTransform,
    ILTransformContext,
)


class StructuredControlFlowCleanupTransform(IILTransform):
    """
    Cleanup pass for structured control flow.

    Current scope:
    - remove truly-empty else-branches (`else {}`) represented as an empty IRBlock.
    """

    def run(self, function: IRFunction, context: ILTransformContext) -> None:
        _ = context
        self._rewrite_container(function.body)

    def _rewrite_container(self, container: IRBlockContainer) -> None:
        for block in container.blocks:
            block.statements = [
                self._rewrite_statement(stmt) for stmt in block.statements
            ]

    def _rewrite_statement(self, statement: IRStatement) -> IRStatement:
        if isinstance(statement, IRIf):
            statement.true_statement = self._rewrite_statement(statement.true_statement)
            if statement.false_statement is not None:
                statement.false_statement = self._rewrite_statement(
                    statement.false_statement
                )
                if self._is_truly_empty_branch(statement.false_statement):
                    statement.false_statement = None
            return statement

        if isinstance(statement, IRBlock):
            statement.statements = [
                self._rewrite_statement(stmt) for stmt in statement.statements
            ]
            return statement

        if isinstance(statement, IRBlockContainer):
            self._rewrite_container(statement)
            return statement

        if isinstance(statement, IRHighLevelSwitch):
            for section in statement.sections:
                section.body = cast(IRBlock, self._rewrite_statement(section.body))
            return statement

        return statement

    @staticmethod
    def _is_truly_empty_branch(statement: IRStatement) -> bool:
        return isinstance(statement, IRBlock) and len(statement.statements) == 0
