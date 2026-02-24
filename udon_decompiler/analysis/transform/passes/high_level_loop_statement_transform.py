from __future__ import annotations

from typing import Optional, cast

from udon_decompiler.analysis.ir.nodes import (
    IRBlock,
    IRBlockContainer,
    IRContainerKind,
    IRFunction,
    IRHighLevelDoWhile,
    IRHighLevelSwitch,
    IRHighLevelWhile,
    IRIf,
    IRJump,
    IRLeave,
    IRStatement,
)
from udon_decompiler.analysis.transform.pass_base import (
    ITransform,
    TransformContext,
)


class HighLevelLoopStatementTransform(ITransform):
    """
    Lift low-level loop containers into explicit high-level loop statements.

    - `ContainerKind.Loop`      -> `while (true)`
    - `ContainerKind.While`     -> `while (condition)`
    - `ContainerKind.DoWhile`   -> `do { ... } while (condition)`
    """

    def run(self, function: IRFunction, context: TransformContext) -> None:
        self._context = context
        self._rewrite_container(function.body)

    def _rewrite_container(self, container: IRBlockContainer) -> None:
        for block in list(container.blocks):
            block.statements = [
                self._rewrite_statement(statement) for statement in block.statements
            ]

    def _rewrite_statement(self, statement: IRStatement) -> IRStatement:
        if isinstance(statement, IRIf):
            statement.true_statement = self._rewrite_statement(statement.true_statement)
            if statement.false_statement is not None:
                statement.false_statement = self._rewrite_statement(
                    statement.false_statement
                )
            return statement

        if isinstance(statement, IRBlock):
            statement.statements = [
                self._rewrite_statement(nested) for nested in statement.statements
            ]
            return statement

        if isinstance(statement, IRHighLevelSwitch):
            for section in statement.sections:
                section.body = cast(IRBlock, self._rewrite_statement(section.body))
            return statement

        if isinstance(statement, IRBlockContainer):
            self._rewrite_container(statement)
            return self._lift_container(statement)

        return statement

    def _lift_container(self, container: IRBlockContainer) -> IRStatement:
        if container.kind == IRContainerKind.LOOP:
            return self._lift_infinite_loop(container)

        if container.kind == IRContainerKind.WHILE:
            while_stmt = self._lift_while(container)
            if while_stmt is not None:
                return while_stmt
            return container

        if container.kind == IRContainerKind.DO_WHILE:
            do_while_stmt = self._lift_do_while(container)
            if do_while_stmt is not None:
                return do_while_stmt
            return container

        return container

    def _lift_infinite_loop(
        self,
        container: IRBlockContainer,
    ) -> IRStatement:
        entry = container.entry_block
        if entry is None:
            return container

        body = IRBlockContainer(
            blocks=list(container.blocks),
            kind=IRContainerKind.BLOCK,
        )
        self._remove_trailing_continue_jump(body, entry)
        return IRHighLevelWhile(
            condition=None,
            body=body,
            continue_target=entry,
            break_target=container,
        )

    def _lift_while(
        self,
        container: IRBlockContainer,
    ) -> Optional[IRStatement]:
        entry = container.entry_block
        if entry is None:
            return None
        if len(entry.statements) != 1:
            return None

        if_inst = entry.statements[0]
        if not isinstance(if_inst, IRIf):
            return None
        if if_inst.false_statement is None:
            return None
        if not self._is_leave_to_container(if_inst.false_statement, container):
            return None

        inline_loop_body_block: Optional[IRBlock] = None
        loop_body_target: Optional[IRBlock] = None

        if isinstance(if_inst.true_statement, IRJump):
            loop_body_target = if_inst.true_statement.target
            if loop_body_target not in container.blocks:
                return None
        elif isinstance(if_inst.true_statement, IRBlock):
            inline_loop_body_block = IRBlock(
                statements=list(if_inst.true_statement.statements),
                start_address=self._next_synthetic_block_address(),
            )
        else:
            return None

        # The condition block becomes a pure continue label anchor.
        entry.statements = []

        body_blocks: list[IRBlock] = []
        if inline_loop_body_block is not None:
            body_blocks.append(inline_loop_body_block)
        elif loop_body_target is not None:
            body_blocks.append(loop_body_target)

        for block in container.blocks[1:]:
            if loop_body_target is not None and block is loop_body_target:
                continue
            body_blocks.append(block)

        # Keep a physical anchor block for gotos that cannot be rendered as continue.
        body_blocks.append(entry)

        body_container = IRBlockContainer(
            blocks=body_blocks,
            kind=IRContainerKind.BLOCK,
        )
        self._remove_trailing_continue_jump(body_container, entry)

        return IRHighLevelWhile(
            condition=if_inst.condition,
            body=body_container,
            continue_target=entry,
            break_target=container,
        )

    def _lift_do_while(
        self,
        container: IRBlockContainer,
    ) -> Optional[IRStatement]:
        entry = container.entry_block
        if entry is None or not container.blocks:
            return None

        condition_block = container.blocks[-1]
        if not condition_block.statements:
            return None

        if_inst = condition_block.statements[-1]
        if not isinstance(if_inst, IRIf):
            return None
        if if_inst.false_statement is None:
            return None
        if not self._is_leave_to_container(if_inst.false_statement, container):
            return None
        if not self._is_jump_to_entry(if_inst.true_statement, entry):
            return None

        body_blocks = list(container.blocks[:-1])
        condition_prefix = condition_block.statements[:-1]

        if condition_prefix:
            body_blocks.append(
                IRBlock(
                    statements=list(condition_prefix),
                    start_address=self._next_synthetic_block_address(),
                )
            )

        # The original condition block becomes a pure continue label anchor.
        condition_block.statements = []
        body_blocks.append(condition_block)

        body_container = IRBlockContainer(
            blocks=body_blocks,
            kind=IRContainerKind.BLOCK,
        )
        self._remove_trailing_continue_jump(body_container, condition_block)

        return IRHighLevelDoWhile(
            condition=if_inst.condition,
            body=body_container,
            continue_target=condition_block,
            break_target=container,
        )

    @staticmethod
    def _is_leave_to_container(
        statement: IRStatement,
        container: IRBlockContainer,
    ) -> bool:
        return (
            isinstance(statement, IRLeave) and statement.target_container is container
        )

    @staticmethod
    def _is_jump_to_entry(
        statement: IRStatement,
        entry: IRBlock,
    ) -> bool:
        if isinstance(statement, IRJump):
            return statement.target is entry
        if isinstance(statement, IRBlock) and len(statement.statements) == 1:
            nested = statement.statements[0]
            return isinstance(nested, IRJump) and nested.target is entry
        return False

    def _next_synthetic_block_address(self) -> int:
        key = "_synthetic_block_addr"
        current = self._context.metadata.get(key)
        if not isinstance(current, int):
            current = -1
        self._context.metadata[key] = current - 1
        return current

    @staticmethod
    def _remove_trailing_continue_jump(
        body: IRBlockContainer,
        continue_target: IRBlock,
    ) -> None:
        for block in reversed(body.blocks):
            if not block.statements:
                continue
            last = block.statements[-1]
            if isinstance(last, IRJump) and last.target is continue_target:
                block.statements.pop()
            break
