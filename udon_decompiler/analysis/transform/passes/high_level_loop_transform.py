from __future__ import annotations

from typing import Optional

from udon_decompiler.analysis.expression_builder import Operator
from udon_decompiler.analysis.ir.nodes import (
    IRBlock,
    IRBlockContainer,
    IRContainerKind,
    IRExpression,
    IRIf,
    IRJump,
    IRLeave,
    IROperatorCallExpression,
    IRStatement,
)
from udon_decompiler.analysis.transform.ir_utils import iter_block_containers
from udon_decompiler.analysis.transform.pass_base import (
    IILTransform,
    ILTransformContext,
)


class HighLevelLoopTransform(IILTransform):
    """
    Convert low-level loop containers into high-level loop kinds.

    Current scope:
    - while
    - do-while

    For-loop specific rewrites are intentionally not implemented yet.
    """

    def run(self, function, context: ILTransformContext) -> None:
        containers = list(iter_block_containers(function))
        for container in containers:
            if container.kind != IRContainerKind.LOOP:
                continue
            if self._match_do_while_loop(container):
                continue
            self._match_while_loop(container)

    def _match_while_loop(self, loop: IRBlockContainer) -> bool:
        """
        Match canonical while-shape produced by earlier CFG structuring:

        entry:
            ...
            if (cond) <loop-body-target>
            leave loop
        """

        entry = loop.entry_block
        if entry is None or len(entry.statements) < 2:
            return False

        if_inst = entry.statements[-2]
        exit_inst = entry.statements[-1]
        if not isinstance(if_inst, IRIf) or if_inst.false_statement is not None:
            return False
        if not isinstance(exit_inst, IRLeave) or exit_inst.target_container is not loop:
            return False
        if self._is_branch_to_entry(if_inst.true_statement, loop):
            return False
        if not self._is_valid_while_true_branch(if_inst.true_statement, loop):
            return False

        if_inst.false_statement = exit_inst
        entry.statements.pop()
        loop.kind = IRContainerKind.WHILE
        return True

    def _match_do_while_loop(self, loop: IRBlockContainer) -> bool:
        """
        Match canonical do-while condition block near loop end:

        block:
            ...
            if (cond) goto entry
            leave loop

        or the swapped variant:

        block:
            ...
            if (cond) leave loop
            goto entry
        """

        entry = loop.entry_block
        if entry is None:
            return False

        for block in reversed(loop.blocks):
            if len(block.statements) < 2:
                continue

            maybe_if = block.statements[-2]
            maybe_last = block.statements[-1]
            if not isinstance(maybe_if, IRIf) or maybe_if.false_statement is not None:
                continue

            if (
                isinstance(maybe_last, IRLeave)
                and maybe_last.target_container is loop
                and self._is_branch_to_entry(maybe_if.true_statement, loop)
            ):
                maybe_if.false_statement = maybe_last
                block.statements.pop()
                loop.kind = IRContainerKind.DO_WHILE
                return True

            if (
                self._is_branch_to_entry(maybe_last, loop)
                and isinstance(maybe_if.true_statement, IRLeave)
                and maybe_if.true_statement.target_container is loop
            ):
                block.statements[-2] = IRIf(
                    condition=self._logic_not(maybe_if.condition),
                    true_statement=maybe_last,
                    false_statement=maybe_if.true_statement,
                )
                block.statements.pop()
                loop.kind = IRContainerKind.DO_WHILE
                return True

        return False

    def _is_valid_while_true_branch(
        self,
        statement: IRStatement,
        loop: IRBlockContainer,
    ) -> bool:
        if isinstance(statement, IRBlock):
            return True
        if isinstance(statement, IRJump):
            return statement.target in loop.blocks
        return False

    def _is_branch_to_entry(
        self,
        statement: IRStatement,
        loop: IRBlockContainer,
    ) -> bool:
        entry = loop.entry_block
        if entry is None:
            return False

        if isinstance(statement, IRJump):
            return statement.target is entry

        # ConditionDetection may wrap a branch in a one-statement block.
        if isinstance(statement, IRBlock) and len(statement.statements) == 1:
            inner = statement.statements[0]
            if isinstance(inner, IRJump):
                return inner.target is entry

        return False

    def _logic_not(self, expression: IRExpression) -> IRExpression:
        if (
            isinstance(expression, IROperatorCallExpression)
            and expression.operator == Operator.UnaryNegation
            and len(expression.arguments) == 1
        ):
            return expression.arguments[0]
        return IROperatorCallExpression(
            arguments=[expression],
            operator=Operator.UnaryNegation,
        )
