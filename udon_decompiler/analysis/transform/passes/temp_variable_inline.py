from __future__ import annotations

from copy import deepcopy

from udon_decompiler.analysis.ir.nodes import (
    IRAssignmentStatement,
    IRBlock,
    IRBlockContainer,
    IRConstructorCallExpression,
    IRExpression,
    IRExpressionStatement,
    IRExternalCallExpression,
    IRIf,
    IROperatorCallExpression,
    IRPropertyAccessExpression,
    IRStatement,
    IRSwitch,
    IRVariableExpression,
)
from udon_decompiler.analysis.transform.ir_utils import iter_block_containers
from udon_decompiler.analysis.transform.pass_base import (
    IILTransform,
    ILTransformContext,
)
from udon_decompiler.models.program import SymbolInfo


class TempVariableInline(IILTransform):
    """Inline adjacent single-use ``__intnl_*`` temporaries within one block."""

    def run(self, function, context: ILTransformContext) -> None:
        visited_blocks: set[int] = set()

        for container in iter_block_containers(function):
            for block in container.blocks:
                self._inline_block(block, visited_blocks)

    def _inline_block(self, block: IRBlock, visited_blocks: set[int]) -> None:
        block_id = id(block)
        if block_id in visited_blocks:
            return
        visited_blocks.add(block_id)

        i = 1
        while i < len(block.statements):
            if self._try_inline_adjacent(block, i):
                i = max(1, i - 1)
                continue
            i += 1

        for statement in block.statements:
            self._inline_nested(statement, visited_blocks)

    def _inline_nested(self, statement: IRStatement, visited_blocks: set[int]) -> None:
        if isinstance(statement, IRBlock):
            self._inline_block(statement, visited_blocks)
            return

        if isinstance(statement, IRBlockContainer):
            for block in statement.blocks:
                self._inline_block(block, visited_blocks)
            return

        if isinstance(statement, IRIf):
            self._inline_nested(statement.true_statement, visited_blocks)
            if statement.false_statement is not None:
                self._inline_nested(statement.false_statement, visited_blocks)

    def _try_inline_adjacent(self, block: IRBlock, index: int) -> bool:
        prev = block.statements[index - 1]
        curr = block.statements[index]

        if not isinstance(prev, IRAssignmentStatement):
            return False

        variable = prev.target
        if not variable.name.startswith(SymbolInfo.INTERNAL_SYMBOL_PREFIX):
            return False

        if self._writes_variable_top_level(curr, variable):
            return False

        if self._count_reads_top_level(curr, variable) != 1:
            return False

        if not self._replace_reads_top_level(curr, variable, prev.value):
            return False

        block.statements.pop(index - 1)
        return True

    def _writes_variable_top_level(self, statement: IRStatement, variable) -> bool:
        return (
            isinstance(statement, IRAssignmentStatement)
            and statement.target == variable
        )

    def _count_reads_top_level(self, statement: IRStatement, variable) -> int:
        if isinstance(statement, IRAssignmentStatement):
            return self._count_expr_reads(statement.value, variable)

        if isinstance(statement, IRExpressionStatement):
            return self._count_expr_reads(statement.expression, variable)

        if isinstance(statement, IRIf):
            return self._count_expr_reads(statement.condition, variable)

        if isinstance(statement, IRSwitch):
            return self._count_expr_reads(statement.index_expression, variable)

        return 0

    def _count_expr_reads(self, expression: IRExpression, variable) -> int:
        if isinstance(expression, IRVariableExpression):
            return 1 if expression.variable == variable else 0

        if isinstance(expression, IROperatorCallExpression):
            return sum(
                self._count_expr_reads(arg, variable) for arg in expression.arguments
            )

        if isinstance(expression, IRExternalCallExpression):
            return sum(
                self._count_expr_reads(arg, variable) for arg in expression.arguments
            )

        if isinstance(expression, IRPropertyAccessExpression):
            return sum(
                self._count_expr_reads(arg, variable) for arg in expression.arguments
            )

        if isinstance(expression, IRConstructorCallExpression):
            return sum(
                self._count_expr_reads(arg, variable) for arg in expression.arguments
            )

        return 0

    def _replace_reads_top_level(
        self,
        statement: IRStatement,
        variable,
        replacement: IRExpression,
    ) -> bool:
        if isinstance(statement, IRAssignmentStatement):
            statement.value, count = self._replace_expr_reads(
                statement.value,
                variable,
                replacement,
            )
            return count == 1

        if isinstance(statement, IRExpressionStatement):
            statement.expression, count = self._replace_expr_reads(
                statement.expression,
                variable,
                replacement,
            )
            return count == 1

        if isinstance(statement, IRIf):
            statement.condition, count = self._replace_expr_reads(
                statement.condition,
                variable,
                replacement,
            )
            return count == 1

        if isinstance(statement, IRSwitch):
            statement.index_expression, count = self._replace_expr_reads(
                statement.index_expression,
                variable,
                replacement,
            )
            return count == 1

        return False

    def _replace_expr_reads(
        self,
        expression: IRExpression,
        variable,
        replacement: IRExpression,
    ) -> tuple[IRExpression, int]:
        if isinstance(expression, IRVariableExpression):
            if expression.variable == variable:
                return deepcopy(replacement), 1
            return expression, 0

        if isinstance(expression, IROperatorCallExpression):
            count = 0
            new_args: list[IRExpression] = []
            for arg in expression.arguments:
                new_arg, c = self._replace_expr_reads(arg, variable, replacement)
                new_args.append(new_arg)
                count += c
            expression.arguments = new_args
            return expression, count

        if isinstance(expression, IRExternalCallExpression):
            count = 0
            new_args: list[IRExpression] = []
            for arg in expression.arguments:
                new_arg, c = self._replace_expr_reads(arg, variable, replacement)
                new_args.append(new_arg)
                count += c
            expression.arguments = new_args
            return expression, count

        if isinstance(expression, IRPropertyAccessExpression):
            count = 0
            new_args: list[IRExpression] = []
            for arg in expression.arguments:
                new_arg, c = self._replace_expr_reads(arg, variable, replacement)
                new_args.append(new_arg)
                count += c
            expression.arguments = new_args
            return expression, count

        if isinstance(expression, IRConstructorCallExpression):
            count = 0
            new_args: list[IRExpression] = []
            for arg in expression.arguments:
                new_arg, c = self._replace_expr_reads(arg, variable, replacement)
                new_args.append(new_arg)
                count += c
            expression.arguments = new_args
            return expression, count

        return expression, 0
