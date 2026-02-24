from __future__ import annotations

from dataclasses import dataclass
from typing import Dict, Iterator, Optional

from udon_decompiler.analysis.ir.nodes import (
    IRBlock,
    IRBlockContainer,
    IRFunction,
    IRHighLevelSwitch,
    IRIf,
    IRJump,
    IRLeave,
    IRReturn,
    IRStatement,
    IRSwitch,
)
from udon_decompiler.analysis.transform.pass_base import (
    ITransform,
    TransformContext,
)


@dataclass
class _SimplifiedJump:
    statement: IRStatement
    changed: bool


class ControlFlowSimplification(ITransform):
    """
    Early CFG simplification on current IR.

    Current scope:
    - simplify branch-to-branch chains
    - replace branch-to-leave with direct leave
    - combine basic blocks where safe
    - remove dead empty blocks
    """

    def run(self, function: IRFunction, context: TransformContext) -> None:
        # Run until fixed point
        for _ in range(32):
            changed = False
            if self._simplify_branch_chains(function, context):
                changed = True
            if self._cleanup_empty_blocks(function, context):
                changed = True
            if not changed:
                break

    def _simplify_branch_chains(
        self,
        function: IRFunction,
        context: TransformContext,
    ) -> bool:
        changed = False
        for container in self._iter_containers(function.body):
            for block in container.blocks:
                for index, statement in enumerate(list(block.statements)):
                    updated, stmt_changed = self._rewrite_statement(statement, context)
                    if stmt_changed:
                        block.statements[index] = updated
                        changed = True
        return changed

    def _rewrite_statement(
        self,
        statement: IRStatement,
        context: TransformContext,
    ) -> tuple[IRStatement, bool]:
        if isinstance(statement, IRJump):
            simplified = self._simplify_jump(statement, context)
            return simplified.statement, simplified.changed

        if isinstance(statement, IRIf):
            true_stmt, true_changed = self._rewrite_statement(
                statement.true_statement,
                context,
            )
            false_changed = False
            false_stmt = statement.false_statement
            if false_stmt is not None:
                false_stmt, false_changed = self._rewrite_statement(
                    false_stmt,
                    context,
                )
            statement.true_statement = true_stmt
            statement.false_statement = false_stmt
            return statement, (true_changed or false_changed)

        if isinstance(statement, IRBlock):
            changed = False
            for index, nested in enumerate(list(statement.statements)):
                updated, nested_changed = self._rewrite_statement(nested, context)
                if nested_changed:
                    statement.statements[index] = updated
                    changed = True
            return statement, changed

        if isinstance(statement, IRSwitch):
            changed = False
            for case_value, target in list(statement.cases.items()):
                resolved_target, target_changed = self._resolve_branch_target(target)
                if target_changed:
                    statement.cases[case_value] = resolved_target
                    changed = True
            if statement.default_target is not None:
                resolved_default, default_changed = self._resolve_branch_target(
                    statement.default_target
                )
                if default_changed:
                    statement.default_target = resolved_default
                    changed = True
            return statement, changed

        if isinstance(statement, IRHighLevelSwitch):
            changed = False
            for section in statement.sections:
                updated_body, body_changed = self._rewrite_statement(
                    section.body,
                    context,
                )
                if body_changed:
                    section.body = updated_body  # type: ignore[assignment]
                    changed = True
            return statement, changed

        if isinstance(statement, IRBlockContainer):
            changed = False
            for block in statement.blocks:
                for index, nested in enumerate(list(block.statements)):
                    updated, nested_changed = self._rewrite_statement(nested, context)
                    if nested_changed:
                        block.statements[index] = updated
                        changed = True
            return statement, changed

        return statement, False

    def _simplify_jump(
        self,
        jump: IRJump,
        context: TransformContext,
    ) -> _SimplifiedJump:
        original_target = jump.target
        target, chain_simplified = self._resolve_branch_target(original_target)
        if chain_simplified:
            context.step("Simplify branch to branch")

        if len(target.statements) == 1 and isinstance(target.statements[0], IRLeave):
            context.step("Replace branch to leave with leave")
            leave = target.statements[0]
            return _SimplifiedJump(
                statement=IRLeave(target_container=leave.target_container),
                changed=True,
            )

        if chain_simplified and target is not original_target:
            return _SimplifiedJump(statement=IRJump(target=target), changed=True)

        return _SimplifiedJump(statement=jump, changed=chain_simplified)

    def _resolve_branch_target(self, target: IRBlock) -> tuple[IRBlock, bool]:
        visited: set[IRBlock] = set()
        changed = False
        current = target

        while (
            len(current.statements) == 1
            and isinstance(current.statements[0], IRJump)
            and current not in visited
        ):
            visited.add(current)
            current = current.statements[0].target
            changed = True

        return current, changed

    def _cleanup_empty_blocks(
        self,
        function: IRFunction,
        context: TransformContext,
    ) -> bool:
        changed = False

        for container in self._iter_containers(function.body):
            while True:
                incoming = self._compute_global_incoming_counts(function.body)
                combined = False
                for block in list(container.blocks):
                    if self._combine_block_with_next(
                        container=container,
                        block=block,
                        incoming=incoming,
                        context=context,
                    ):
                        changed = True
                        combined = True
                        break
                if not combined:
                    break

            incoming = self._compute_global_incoming_counts(function.body)
            entry = container.entry_block
            kept_blocks: list[IRBlock] = []
            removed = False
            for block in container.blocks:
                if block is entry:
                    kept_blocks.append(block)
                    continue
                if incoming.get(block, 0) == 0:
                    removed = True
                    continue
                kept_blocks.append(block)
            if removed:
                context.step("Remove dead blocks")
                container.blocks = kept_blocks
                changed = True

        return changed

    def _combine_block_with_next(
        self,
        container: IRBlockContainer,
        block: IRBlock,
        incoming: Dict[IRBlock, int],
        context: TransformContext,
    ) -> bool:
        if block not in container.blocks:
            return False
        if not block.statements:
            return False

        if len(block.statements) > 1 and self._statement_may_branch(
            block.statements[-2]
        ):
            return False

        terminal = block.statements[-1]
        if not isinstance(terminal, IRJump):
            return False

        target_block = terminal.target
        if target_block not in container.blocks:
            return False
        if target_block is block:
            return False
        if incoming.get(target_block, 0) != 1:
            return False

        context.step("Combine block with successor")
        block.statements.pop()
        block.statements.extend(target_block.statements)
        target_block.statements.clear()
        return True

    @staticmethod
    def _statement_may_branch(statement: IRStatement) -> bool:
        if isinstance(statement, (IRIf, IRJump, IRLeave, IRReturn, IRSwitch)):
            return True
        if isinstance(statement, IRHighLevelSwitch):
            return True
        if isinstance(statement, IRBlockContainer):
            return True
        return False

    def _compute_global_incoming_counts(
        self,
        body: IRBlockContainer,
    ) -> Dict[IRBlock, int]:
        counts: Dict[IRBlock, int] = {}
        for container in self._iter_containers(body):
            for block in container.blocks:
                counts.setdefault(block, 0)

        for container in self._iter_containers(body):
            for block in container.blocks:
                for statement in block.statements:
                    self._count_statement_targets(statement, counts)
        return counts

    def _count_statement_targets(
        self,
        statement: IRStatement,
        counts: Dict[IRBlock, int],
    ) -> None:
        if isinstance(statement, IRJump):
            counts[statement.target] = counts.get(statement.target, 0) + 1
            return

        if isinstance(statement, IRIf):
            self._count_statement_targets(statement.true_statement, counts)
            if statement.false_statement is not None:
                self._count_statement_targets(statement.false_statement, counts)
            return

        if isinstance(statement, IRSwitch):
            for target in statement.cases.values():
                counts[target] = counts.get(target, 0) + 1
            if statement.default_target is not None:
                target = statement.default_target
                counts[target] = counts.get(target, 0) + 1
            return

        if isinstance(statement, IRHighLevelSwitch):
            for section in statement.sections:
                self._count_statement_targets(section.body, counts)
            return

        if isinstance(statement, IRBlock):
            for nested in statement.statements:
                self._count_statement_targets(nested, counts)
            return

        if isinstance(statement, IRBlockContainer):
            for block in statement.blocks:
                for nested in block.statements:
                    self._count_statement_targets(nested, counts)

    def _iter_containers(
        self,
        root: IRBlockContainer,
    ) -> Iterator[IRBlockContainer]:
        visited: set[int] = set()
        stack: list[IRBlockContainer] = [root]

        while stack:
            container = stack.pop()
            key = id(container)
            if key in visited:
                continue
            visited.add(key)
            yield container

            for block in container.blocks:
                for statement in block.statements:
                    stack.extend(self._nested_containers_in_statement(statement))

    def _nested_containers_in_statement(
        self,
        statement: IRStatement,
    ) -> list[IRBlockContainer]:
        if isinstance(statement, IRBlockContainer):
            result = [statement]
            for block in statement.blocks:
                for nested in block.statements:
                    result.extend(self._nested_containers_in_statement(nested))
            return result

        if isinstance(statement, IRIf):
            result = self._nested_containers_in_statement(statement.true_statement)
            if statement.false_statement is not None:
                result.extend(
                    self._nested_containers_in_statement(statement.false_statement)
                )
            return result

        if isinstance(statement, IRBlock):
            result: list[IRBlockContainer] = []
            for nested in statement.statements:
                result.extend(self._nested_containers_in_statement(nested))
            return result

        if isinstance(statement, IRHighLevelSwitch):
            result: list[IRBlockContainer] = []
            for section in statement.sections:
                result.extend(self._nested_containers_in_statement(section.body))
            return result

        return []
