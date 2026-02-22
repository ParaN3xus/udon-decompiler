from __future__ import annotations

from typing import Optional, cast

from udon_decompiler.analysis.ir.nodes import (
    IRBlock,
    IRBlockContainer,
    IRFunction,
    IRHighLevelDoWhile,
    IRHighLevelSwitch,
    IRHighLevelSwitchSection,
    IRHighLevelWhile,
    IRIf,
    IRJump,
    IRLeave,
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
        self._hoist_shared_switch_default_tails(container)

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

        if isinstance(statement, IRHighLevelWhile):
            statement.body = cast(
                IRBlockContainer,
                self._rewrite_statement(statement.body),
            )
            self._simplify_linear_if_goto_diamonds(statement.body)
            return statement

        if isinstance(statement, IRHighLevelDoWhile):
            statement.body = cast(
                IRBlockContainer,
                self._rewrite_statement(statement.body),
            )
            self._simplify_linear_if_goto_diamonds(statement.body)
            return statement

        if isinstance(statement, IRHighLevelSwitch):
            for section in statement.sections:
                section.body = cast(IRBlock, self._rewrite_statement(section.body))
            return statement

        return statement

    @staticmethod
    def _is_truly_empty_branch(statement: IRStatement) -> bool:
        return isinstance(statement, IRBlock) and len(statement.statements) == 0

    def _simplify_linear_if_goto_diamonds(
        self,
        body: IRBlockContainer,
    ) -> None:
        """
        Simplify linear shape:
            A: if (c) goto T; goto U;
            T: ...; goto U;
            U: ...
        into:
            A: if (c) { ... }
            U: ...
        """
        changed = True
        while changed:
            changed = False
            for block_index, block in enumerate(list(body.blocks)):
                if block_index >= len(body.blocks):
                    break
                if len(block.statements) < 2:
                    continue

                if_stmt = block.statements[-2]
                after_if = block.statements[-1]
                if not isinstance(if_stmt, IRIf) or if_stmt.false_statement is not None:
                    continue
                if not isinstance(after_if, IRJump):
                    continue

                true_target = self._as_jump_target(if_stmt.true_statement)
                if true_target is None:
                    continue
                false_target = after_if.target

                # Require strict linearized diamond to keep semantics obvious.
                if block_index + 2 >= len(body.blocks):
                    continue
                if body.blocks[block_index + 1] is not true_target:
                    continue
                if body.blocks[block_index + 2] is not false_target:
                    continue

                true_statements = list(true_target.statements)
                if true_statements:
                    last_true = true_statements[-1]
                    if isinstance(last_true, IRJump):
                        if last_true.target is not false_target:
                            continue
                        true_statements.pop()
                    elif isinstance(last_true, (IRIf, IRLeave)):
                        continue

                if_stmt.true_statement = IRBlock(
                    statements=true_statements,
                    start_address=true_target.start_address,
                )

                block.statements.pop()  # remove goto U
                if true_target in body.blocks:
                    body.blocks.remove(true_target)

                changed = True
                break

    def _hoist_shared_switch_default_tails(
        self,
        container: IRBlockContainer,
    ) -> None:
        """
        Deduplicate lowered range-guarded switch tails.

        Pattern:
            ...;
            if (guard) { switch (...) { default: <tail>; } }
            <tail>;

        where `default` body equals the trailing statements in the same block.
        Rewrite into:
            ...;
            if (guard) { switch (...) { default: goto <tail_block>; } }
            <tail_block>: <tail>;
        """
        changed = True
        while changed:
            changed = False
            for block in list(container.blocks):
                for stmt_index, statement in enumerate(block.statements):
                    if not isinstance(statement, IRIf):
                        continue
                    if statement.false_statement is not None:
                        continue

                    switch_stmt = self._extract_single_switch(statement.true_statement)
                    if switch_stmt is None:
                        continue

                    default_section = self._find_default_section(switch_stmt)
                    if default_section is None:
                        continue

                    tail = block.statements[stmt_index + 1 :]
                    if not tail:
                        continue
                    if not self._statement_list_equal(
                        default_section.body.statements,
                        tail,
                    ):
                        continue

                    existing = self._find_block_by_start_address(
                        container,
                        default_section.body.start_address,
                    )
                    if existing is not None and existing is not block:
                        continue

                    shared_block = IRBlock(
                        statements=tail,
                        start_address=default_section.body.start_address,
                    )
                    block.statements = block.statements[: stmt_index + 1]

                    insert_at = container.blocks.index(block) + 1
                    container.blocks.insert(insert_at, shared_block)

                    default_section.body = IRBlock(
                        statements=[IRJump(target=shared_block)],
                        start_address=default_section.body.start_address,
                    )

                    changed = True
                    break
                if changed:
                    break

    @staticmethod
    def _as_jump_target(statement: IRStatement) -> Optional[IRBlock]:
        if isinstance(statement, IRJump):
            return statement.target
        if isinstance(statement, IRBlock) and len(statement.statements) == 1:
            nested = statement.statements[0]
            if isinstance(nested, IRJump):
                return nested.target
        return None

    @staticmethod
    def _extract_single_switch(statement: IRStatement) -> Optional[IRHighLevelSwitch]:
        if not isinstance(statement, IRBlock):
            return None
        if len(statement.statements) != 1:
            return None
        only_statement = statement.statements[0]
        if not isinstance(only_statement, IRHighLevelSwitch):
            return None
        return only_statement

    @staticmethod
    def _find_default_section(
        switch_stmt: IRHighLevelSwitch,
    ) -> Optional[IRHighLevelSwitchSection]:
        for section in switch_stmt.sections:
            if section.is_default:
                return section
        return None

    @staticmethod
    def _statement_list_equal(
        left: list[IRStatement],
        right: list[IRStatement],
    ) -> bool:
        if len(left) != len(right):
            return False
        return all(l_stmt == r_stmt for l_stmt, r_stmt in zip(left, right))

    @staticmethod
    def _find_block_by_start_address(
        container: IRBlockContainer,
        start_address: int,
    ) -> Optional[IRBlock]:
        for block in container.blocks:
            if block.start_address == start_address:
                return block
        return None
