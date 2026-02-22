from __future__ import annotations

from typing import Optional

from udon_decompiler.analysis.ir.nodes import (
    IRBlock,
    IRBlockContainer,
    IRContainerKind,
    IRFunction,
    IRHighLevelSwitch,
    IRIf,
    IRJump,
    IRLeave,
    IRReturn,
    IRStatement,
)
from udon_decompiler.analysis.transform.pass_base import (
    IILTransform,
    ILTransformContext,
)


class CollectLabelUsage(IILTransform):
    """
    Collect label usage info for codegen.

    After this pass:
    - IRBlock.should_emit_label marks whether `<label>:` must be emitted.
    - IRBlockContainer.should_emit_exit_label marks whether exit label must be emitted.
    """

    def run(self, function: IRFunction, context: ILTransformContext) -> None:
        _ = context
        self._reset_flags(function.body)
        self._analyze_container(
            container=function.body,
            break_target=None,
            function_body=function.body,
        )

    def _reset_flags(self, container: IRBlockContainer) -> None:
        container.should_emit_exit_label = False
        for block in container.blocks:
            block.should_emit_label = False
            for statement in block.statements:
                self._reset_in_statement(statement)

    def _reset_in_statement(self, statement: IRStatement) -> None:
        if isinstance(statement, IRIf):
            self._reset_in_statement(statement.true_statement)
            if statement.false_statement is not None:
                self._reset_in_statement(statement.false_statement)
            return

        if isinstance(statement, IRHighLevelSwitch):
            for section in statement.sections:
                section.body.should_emit_label = False
                for nested in section.body.statements:
                    self._reset_in_statement(nested)
            return

        if isinstance(statement, IRBlock):
            statement.should_emit_label = False
            for nested in statement.statements:
                self._reset_in_statement(nested)
            return

        if isinstance(statement, IRBlockContainer):
            self._reset_flags(statement)

    def _analyze_container(
        self,
        container: IRBlockContainer,
        break_target: Optional[IRBlockContainer],
        function_body: IRBlockContainer,
    ) -> None:
        if self._analyze_while_like(
            container=container,
            break_target=break_target,
            function_body=function_body,
        ):
            return

        if self._analyze_do_while_like(
            container=container,
            break_target=break_target,
            function_body=function_body,
        ):
            return

        for index, block in enumerate(container.blocks):
            next_block = (
                container.blocks[index + 1]
                if index + 1 < len(container.blocks)
                else None
            )
            for stmt_index, statement in enumerate(block.statements):
                statement_next_block = (
                    next_block if stmt_index == len(block.statements) - 1 else None
                )
                self._analyze_statement(
                    statement=statement,
                    current_container=container,
                    next_block=statement_next_block,
                    break_target=break_target,
                    function_body=function_body,
                )

    def _analyze_while_like(
        self,
        container: IRBlockContainer,
        break_target: Optional[IRBlockContainer],
        function_body: IRBlockContainer,
    ) -> bool:
        if container.kind != IRContainerKind.WHILE:
            return False

        entry = container.entry_block
        if entry is None or len(container.blocks) != 1 or not entry.statements:
            return False

        terminal = entry.statements[-1]
        if not isinstance(terminal, IRIf):
            return False
        if terminal.false_statement is None:
            return False
        if not self._is_leave_to_container(terminal.false_statement, container):
            return False
        if not isinstance(terminal.true_statement, IRBlock):
            return False

        body_statements = entry.statements[:-1] + terminal.true_statement.statements
        for statement in body_statements:
            self._analyze_statement(
                statement=statement,
                current_container=container,
                next_block=None,
                break_target=container,
                function_body=function_body,
            )
        return True

    def _analyze_do_while_like(
        self,
        container: IRBlockContainer,
        break_target: Optional[IRBlockContainer],
        function_body: IRBlockContainer,
    ) -> bool:
        _ = break_target
        if container.kind != IRContainerKind.DO_WHILE:
            return False

        entry = container.entry_block
        if entry is None or len(container.blocks) != 1 or not entry.statements:
            return False

        terminal = entry.statements[-1]
        if not isinstance(terminal, IRIf):
            return False
        if terminal.false_statement is None:
            return False
        if not self._is_leave_to_container(terminal.false_statement, container):
            return False
        if not self._is_branch_to_entry(terminal.true_statement, container):
            return False

        for statement in entry.statements[:-1]:
            self._analyze_statement(
                statement=statement,
                current_container=container,
                next_block=None,
                break_target=container,
                function_body=function_body,
            )
        return True

    def _analyze_statement(
        self,
        statement: IRStatement,
        current_container: Optional[IRBlockContainer],
        next_block: Optional[IRBlock],
        break_target: Optional[IRBlockContainer],
        function_body: IRBlockContainer,
    ) -> None:
        if isinstance(statement, IRIf):
            self._analyze_branch_statement(
                statement=statement.true_statement,
                current_container=current_container,
                break_target=break_target,
                function_body=function_body,
            )
            if statement.false_statement is not None:
                self._analyze_branch_statement(
                    statement=statement.false_statement,
                    current_container=current_container,
                    break_target=break_target,
                    function_body=function_body,
                )
            return

        if isinstance(statement, IRHighLevelSwitch):
            for section in statement.sections:
                for nested in section.body.statements:
                    self._analyze_statement(
                        statement=nested,
                        current_container=current_container,
                        next_block=None,
                        break_target=None,
                        function_body=function_body,
                    )
            return

        if isinstance(statement, IRJump):
            if (
                break_target is not None
                and break_target.entry_block is not None
                and statement.target is break_target.entry_block
            ):
                return
            if next_block is not None and statement.target is next_block:
                return
            statement.target.should_emit_label = True
            return

        if isinstance(statement, IRLeave):
            if statement.target_container is function_body:
                return
            if break_target is not None and statement.target_container is break_target:
                return
            statement.target_container.should_emit_exit_label = True
            return

        if isinstance(statement, IRReturn):
            return

        if isinstance(statement, IRBlock):
            for nested in statement.statements:
                self._analyze_statement(
                    statement=nested,
                    current_container=current_container,
                    next_block=None,
                    break_target=break_target,
                    function_body=function_body,
                )
            return

        if isinstance(statement, IRBlockContainer):
            self._analyze_container(
                container=statement,
                break_target=break_target,
                function_body=function_body,
            )

    def _analyze_branch_statement(
        self,
        statement: IRStatement,
        current_container: Optional[IRBlockContainer],
        break_target: Optional[IRBlockContainer],
        function_body: IRBlockContainer,
    ) -> None:
        if (
            isinstance(statement, IRBlock)
            and current_container is not None
            and statement in current_container.blocks
        ):
            statement.should_emit_label = True
            return

        self._analyze_statement(
            statement=statement,
            current_container=current_container,
            next_block=None,
            break_target=break_target,
            function_body=function_body,
        )

    @staticmethod
    def _is_leave_to_container(
        statement: IRStatement,
        container: IRBlockContainer,
    ) -> bool:
        return (
            isinstance(statement, IRLeave)
            and statement.target_container is container
        )

    @staticmethod
    def _is_branch_to_entry(
        statement: IRStatement,
        container: IRBlockContainer,
    ) -> bool:
        entry = container.entry_block
        if entry is None:
            return False

        if isinstance(statement, IRJump):
            return statement.target is entry

        if isinstance(statement, IRBlock) and len(statement.statements) == 1:
            nested = statement.statements[0]
            return isinstance(nested, IRJump) and nested.target is entry

        return False
