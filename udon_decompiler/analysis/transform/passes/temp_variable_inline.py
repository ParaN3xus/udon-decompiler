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
    IRHighLevelDoWhile,
    IRHighLevelSwitch,
    IRHighLevelWhile,
    IRIf,
    IRJump,
    IRLeave,
    IROperatorCallExpression,
    IRPropertyAccessExpression,
    IRReturn,
    IRStatement,
    IRSwitch,
    IRVariableExpression,
)
from udon_decompiler.analysis.transform.ir_utils import (
    get_block_terminator,
    iter_block_containers,
    iter_block_targets,
)
from udon_decompiler.analysis.transform.pass_base import (
    IILTransform,
    ILTransformContext,
)
from udon_decompiler.models.program import SymbolInfo


class TempVariableInline(IILTransform):
    """Inline adjacent single-use ``__intnl_*`` temporaries within one block."""

    def run(self, function, context: ILTransformContext) -> None:
        self._function = function
        self._successor_cache: dict[int, dict[IRBlock, list[IRBlock]]] = {}
        visited_blocks: set[int] = set()

        for container in iter_block_containers(function):
            for block in container.blocks:
                self._inline_block(block, container, visited_blocks)

    def _inline_block(
        self,
        block: IRBlock,
        container: IRBlockContainer,
        visited_blocks: set[int],
    ) -> None:
        block_id = id(block)
        if block_id in visited_blocks:
            return
        visited_blocks.add(block_id)

        i = 1
        while i < len(block.statements):
            if self._try_inline_adjacent(block, container, i):
                i = max(1, i - 1)
                continue
            i += 1

        for statement in block.statements:
            self._inline_nested(statement, container, visited_blocks)

    def _inline_nested(
        self,
        statement: IRStatement,
        container: IRBlockContainer,
        visited_blocks: set[int],
    ) -> None:
        if isinstance(statement, IRBlock):
            self._inline_block(statement, container, visited_blocks)
            return

        if isinstance(statement, IRBlockContainer):
            for block in statement.blocks:
                self._inline_block(block, statement, visited_blocks)
            return

        if isinstance(statement, IRIf):
            self._inline_nested(
                statement.true_statement,
                container,
                visited_blocks,
            )
            if statement.false_statement is not None:
                self._inline_nested(
                    statement.false_statement,
                    container,
                    visited_blocks,
                )

    def _try_inline_adjacent(
        self,
        block: IRBlock,
        container: IRBlockContainer,
        index: int,
    ) -> bool:
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

        # Keep the defining assignment when its value can flow to a read in
        # another block before any redefinition.
        if self._definition_flows_to_other_block_read(
            container=container,
            block=block,
            index=index,
            variable=variable,
        ):
            return False

        if not self._replace_reads_top_level(curr, variable, prev.value):
            return False

        block.statements.pop(index - 1)
        return True

    def _definition_flows_to_other_block_read(
        self,
        container: IRBlockContainer,
        block: IRBlock,
        index: int,
        variable,
    ) -> bool:
        # First scan tail in current block.
        for statement in block.statements[index + 1 :]:
            if self._count_reads_in_statement(statement, variable) > 0:
                return True
            if self._count_writes_in_statement(statement, variable) > 0:
                return False

        # Then follow CFG edges to detect reads before next write.
        successors = self._get_successor_map(container)
        worklist = list(successors.get(block, []))
        visited: set[IRBlock] = set()

        while worklist:
            current = worklist.pop()
            if current in visited:
                continue
            visited.add(current)

            killed = False
            for statement in current.statements:
                if self._count_reads_in_statement(statement, variable) > 0:
                    return True
                if self._count_writes_in_statement(statement, variable) > 0:
                    killed = True
                    break

            if not killed:
                worklist.extend(successors.get(current, []))

        return False

    def _get_successor_map(
        self,
        container: IRBlockContainer,
    ) -> dict[IRBlock, list[IRBlock]]:
        key = id(container)
        cached = self._successor_cache.get(key)
        if cached is not None:
            return cached

        successors: dict[IRBlock, list[IRBlock]] = {}
        block_set = set(container.blocks)

        for index, block in enumerate(container.blocks):
            succ_list: list[IRBlock] = []
            next_block = (
                container.blocks[index + 1]
                if index + 1 < len(container.blocks)
                else None
            )
            terminator = get_block_terminator(block)

            if terminator is None:
                if next_block is not None:
                    succ_list.append(next_block)
                successors[block] = succ_list
                continue

            for target in iter_block_targets(terminator):
                if target in block_set and target not in succ_list:
                    succ_list.append(target)

            if isinstance(terminator, IRIf) and terminator.false_statement is None:
                if next_block is not None and next_block not in succ_list:
                    succ_list.append(next_block)
            elif isinstance(terminator, IRSwitch) and terminator.default_target is None:
                if next_block is not None and next_block not in succ_list:
                    succ_list.append(next_block)

            successors[block] = succ_list

        self._successor_cache[key] = successors
        return successors

    def _count_reads_in_statement(self, statement: IRStatement, variable) -> int:
        if isinstance(statement, IRAssignmentStatement):
            return self._count_expr_reads(statement.value, variable)

        if isinstance(statement, IRExpressionStatement):
            return self._count_expr_reads(statement.expression, variable)

        if isinstance(statement, IRIf):
            count = self._count_expr_reads(statement.condition, variable)
            count += self._count_reads_in_statement(statement.true_statement, variable)
            if statement.false_statement is not None:
                count += self._count_reads_in_statement(
                    statement.false_statement,
                    variable,
                )
            return count

        if isinstance(statement, IRSwitch):
            return self._count_expr_reads(statement.index_expression, variable)

        if isinstance(statement, IRBlock):
            count = 0
            for nested in statement.statements:
                count += self._count_reads_in_statement(nested, variable)
            return count

        if isinstance(statement, IRBlockContainer):
            count = 0
            for nested_block in statement.blocks:
                count += self._count_reads_in_statement(nested_block, variable)
            return count

        if isinstance(statement, IRHighLevelSwitch):
            count = self._count_expr_reads(statement.index_expression, variable)
            for section in statement.sections:
                count += self._count_reads_in_statement(section.body, variable)
            return count

        if isinstance(statement, (IRHighLevelWhile, IRHighLevelDoWhile)):
            return self._count_reads_in_statement(statement.body, variable)

        if isinstance(statement, (IRJump, IRLeave, IRReturn)):
            return 0

        return 0

    def _count_writes_in_statement(self, statement: IRStatement, variable) -> int:
        if isinstance(statement, IRAssignmentStatement):
            return 1 if statement.target == variable else 0

        if isinstance(statement, IRIf):
            count = self._count_writes_in_statement(statement.true_statement, variable)
            if statement.false_statement is not None:
                count += self._count_writes_in_statement(
                    statement.false_statement,
                    variable,
                )
            return count

        if isinstance(statement, IRBlock):
            count = 0
            for nested in statement.statements:
                count += self._count_writes_in_statement(nested, variable)
            return count

        if isinstance(statement, IRBlockContainer):
            count = 0
            for nested_block in statement.blocks:
                count += self._count_writes_in_statement(nested_block, variable)
            return count

        if isinstance(statement, IRHighLevelSwitch):
            count = 0
            for section in statement.sections:
                count += self._count_writes_in_statement(section.body, variable)
            return count

        if isinstance(statement, (IRHighLevelWhile, IRHighLevelDoWhile)):
            return self._count_writes_in_statement(statement.body, variable)

        return 0

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
