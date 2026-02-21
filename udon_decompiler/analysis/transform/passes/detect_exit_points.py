from __future__ import annotations

from dataclasses import dataclass
from typing import Callable, Optional

from udon_decompiler.analysis.ir.nodes import (
    IRBlock,
    IRBlockContainer,
    IRFunction,
    IRIf,
    IRJump,
    IRLeave,
    IRStatement,
)
from udon_decompiler.analysis.transform.pass_base import (
    IILTransform,
    ILTransformContext,
)


class _ExitSentinel:
    pass


EXIT_NOT_YET_DETERMINED = _ExitSentinel()
NO_EXIT = _ExitSentinel()


@dataclass
class _ExitOccurrence:
    statement: IRStatement
    replace: Callable[[IRStatement], None]


class DetectExitPoints(IILTransform):
    """Port of ILSpy DetectExitPoints, adapted to the current Python IR."""

    def __init__(self, can_introduce_exit_for_return: bool) -> None:
        self.can_introduce_exit_for_return = can_introduce_exit_for_return

        self._function_body: Optional[IRBlockContainer] = None
        self._current_container: Optional[IRBlockContainer] = None
        self._current_exit: object = NO_EXIT
        self._potential_exits: Optional[list[_ExitOccurrence]] = None

        self._descendant_block_cache: dict[int, set[IRBlock]] = {}

    def run(self, function: IRFunction, context: ILTransformContext) -> None:
        self._function_body = function.body
        self._current_container = None
        self._current_exit = NO_EXIT
        self._potential_exits = None
        self._descendant_block_cache = {}

        self._visit_container(
            container=function.body,
            parent_block=None,
            parent_index=None,
        )

    @staticmethod
    def compatible_exit_instruction(exit1: object, exit2: object) -> bool:
        if type(exit1) is not type(exit2):
            return False

        if isinstance(exit1, IRJump) and isinstance(exit2, IRJump):
            return exit1.target is exit2.target

        if isinstance(exit1, IRLeave) and isinstance(exit2, IRLeave):
            return exit1.target_container is exit2.target_container

        return False

    def _visit_container(
        self,
        container: IRBlockContainer,
        parent_block: Optional[IRBlock],
        parent_index: Optional[int],
    ) -> None:
        old_exit = self._current_exit
        old_container = self._current_container
        old_potential_exits = self._potential_exits

        this_exit = self._get_exit_after_statement(parent_block, parent_index)

        self._current_exit = this_exit
        self._current_container = container
        self._potential_exits = (
            [] if this_exit is EXIT_NOT_YET_DETERMINED else None
        )

        for block in container.blocks:
            self._visit_block(block)

        if (
            this_exit is EXIT_NOT_YET_DETERMINED
            and self._potential_exits is not None
            and self._potential_exits
            and parent_block is not None
            and parent_index is not None
        ):
            chosen_exit = self._choose_exit(self._potential_exits)
            for occurrence in self._potential_exits:
                if self.compatible_exit_instruction(chosen_exit, occurrence.statement):
                    occurrence.replace(IRLeave(target_container=container))

            parent_block.statements.insert(
                parent_index + 1,
                self._clone_exit_statement(chosen_exit),
            )

        self._current_exit = old_exit
        self._current_container = old_container
        self._potential_exits = old_potential_exits

    def _visit_block(self, block: IRBlock) -> None:
        index = 0
        while index < len(block.statements):
            statement = block.statements[index]

            def replace_current(new_statement: IRStatement, i: int = index) -> None:
                block.statements[i] = new_statement

            self._visit_statement(
                statement=statement,
                replace_statement=replace_current,
                parent_block=block,
                parent_index=index,
            )
            index += 1

    def _visit_statement(
        self,
        statement: IRStatement,
        replace_statement: Callable[[IRStatement], None],
        parent_block: Optional[IRBlock],
        parent_index: Optional[int],
    ) -> None:
        if isinstance(statement, IRBlockContainer):
            self._visit_container(statement, parent_block, parent_index)
            return

        if isinstance(statement, IRIf):
            self._visit_statement(
                statement=statement.true_statement,
                replace_statement=lambda new_stmt: setattr(
                    statement,
                    "true_statement",
                    new_stmt,
                ),
                parent_block=None,
                parent_index=None,
            )
            if statement.false_statement is not None:
                self._visit_statement(
                    statement=statement.false_statement,
                    replace_statement=lambda new_stmt: setattr(
                        statement,
                        "false_statement",
                        new_stmt,
                    ),
                    parent_block=None,
                    parent_index=None,
                )
            return

        if isinstance(statement, IRJump):
            if self._current_container is None:
                return

            if not self._is_block_descendant_of_container(
                statement.target,
                self._current_container,
            ):
                self._handle_exit(statement, replace_statement)
            return

        if isinstance(statement, IRLeave):
            self._handle_exit(statement, replace_statement)

    def _handle_exit(
        self,
        statement: IRStatement,
        replace_statement: Callable[[IRStatement], None],
    ) -> None:
        if self._current_container is None:
            return

        if self._current_exit is EXIT_NOT_YET_DETERMINED:
            if (
                self._potential_exits is not None
                and self._can_introduce_as_exit(statement)
            ):
                self._potential_exits.append(
                    _ExitOccurrence(statement=statement, replace=replace_statement)
                )
            return

        if self.compatible_exit_instruction(statement, self._current_exit):
            replace_statement(IRLeave(target_container=self._current_container))

    def _can_introduce_as_exit(self, statement: IRStatement) -> bool:
        if self._current_container is None:
            return False

        if self._container_has_leave_to_self(self._current_container):
            return False

        if isinstance(statement, IRLeave) and self._is_function_return_leave(statement):
            return self.can_introduce_exit_for_return

        return isinstance(statement, (IRJump, IRLeave))

    def _container_has_leave_to_self(self, container: IRBlockContainer) -> bool:
        for block in container.blocks:
            for statement in block.statements:
                if self._statement_has_leave_target(statement, container):
                    return True
        return False

    def _statement_has_leave_target(
        self,
        statement: IRStatement,
        target_container: IRBlockContainer,
    ) -> bool:
        if isinstance(statement, IRLeave):
            return statement.target_container is target_container

        if isinstance(statement, IRIf):
            if self._statement_has_leave_target(
                statement.true_statement,
                target_container,
            ):
                return True
            if statement.false_statement is not None and (
                self._statement_has_leave_target(
                    statement.false_statement,
                    target_container,
                )
            ):
                return True
            return False

        if isinstance(statement, IRBlockContainer):
            for block in statement.blocks:
                for nested in block.statements:
                    if self._statement_has_leave_target(nested, target_container):
                        return True

        return False

    def _is_function_return_leave(self, statement: IRLeave) -> bool:
        if self._function_body is None:
            return False
        return statement.target_container is self._function_body

    def _get_exit_after_statement(
        self,
        parent_block: Optional[IRBlock],
        parent_index: Optional[int],
    ) -> object:
        if parent_block is None or parent_index is None:
            return NO_EXIT

        next_index = parent_index + 1
        if next_index < len(parent_block.statements):
            return parent_block.statements[next_index]

        return EXIT_NOT_YET_DETERMINED

    def _clone_exit_statement(self, statement: object) -> IRStatement:
        if isinstance(statement, IRJump):
            return IRJump(target=statement.target)
        if isinstance(statement, IRLeave):
            return IRLeave(target_container=statement.target_container)
        raise TypeError("Exit statement must be IRJump or IRLeave")

    def _choose_exit(self, exits: list[_ExitOccurrence]) -> IRStatement:
        first = exits[0].statement
        if isinstance(first, IRLeave) and self._is_function_return_leave(first):
            for occurrence in exits[1:]:
                candidate = occurrence.statement
                if not (
                    isinstance(candidate, IRLeave)
                    and self._is_function_return_leave(candidate)
                ):
                    return candidate
        return first

    def _is_block_descendant_of_container(
        self,
        block: IRBlock,
        container: IRBlockContainer,
    ) -> bool:
        cache_key = id(container)
        descendants = self._descendant_block_cache.get(cache_key)
        if descendants is None:
            descendants = self._collect_descendant_blocks(container)
            self._descendant_block_cache[cache_key] = descendants
        return block in descendants

    def _collect_descendant_blocks(self, container: IRBlockContainer) -> set[IRBlock]:
        descendants: set[IRBlock] = set()

        def visit_container(current: IRBlockContainer) -> None:
            for block in current.blocks:
                descendants.add(block)
                for statement in block.statements:
                    if isinstance(statement, IRBlockContainer):
                        visit_container(statement)
                    elif isinstance(statement, IRIf):
                        visit_statement(statement.true_statement)
                        if statement.false_statement is not None:
                            visit_statement(statement.false_statement)

        def visit_statement(statement: IRStatement) -> None:
            if isinstance(statement, IRBlockContainer):
                visit_container(statement)
                return
            if isinstance(statement, IRIf):
                visit_statement(statement.true_statement)
                if statement.false_statement is not None:
                    visit_statement(statement.false_statement)

        visit_container(container)
        return descendants
