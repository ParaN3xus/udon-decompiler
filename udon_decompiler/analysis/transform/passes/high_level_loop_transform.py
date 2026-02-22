from __future__ import annotations

from dataclasses import dataclass
from typing import Optional

from udon_decompiler.analysis.expression_builder import Operator
from udon_decompiler.analysis.ir.nodes import (
    IRBlock,
    IRBlockContainer,
    IRContainerKind,
    IRExpression,
    IRFunction,
    IRIf,
    IRJump,
    IRLeave,
    IROperatorCallExpression,
    IRReturn,
    IRStatement,
    IRSwitch,
)
from udon_decompiler.analysis.transform.ir_utils import iter_block_containers
from udon_decompiler.analysis.transform.pass_base import (
    IILTransform,
    ILTransformContext,
)


@dataclass
class _DoWhileMatch:
    conditions: list[IRIf]
    exit_statement: IRStatement
    swap_branches: bool
    split: bool
    original_block: IRBlock
    condition_block: IRBlock
    unwrap: bool
    unwrap_return_condition: Optional[IRIf]
    unwrap_leave_statement: Optional[IRStatement]
    nested_condition_block: Optional[IRBlock]


class HighLevelLoopTransform(IILTransform):
    """
    Port of ILSpy HighLevelLoopTransform (while/do-while parts).

    This pass upgrades `IRContainerKind.LOOP` to:
    - `IRContainerKind.WHILE`
    - `IRContainerKind.DO_WHILE`
    """

    def run(self, function: IRFunction, context: ILTransformContext) -> None:
        self._context = context
        self._function_body = function.body

        containers = list(iter_block_containers(function))
        for container in containers:
            if container.kind != IRContainerKind.LOOP:
                continue
            if self._match_while_loop(container):
                continue
            self._match_do_while_loop(container)

    def _match_while_loop(self, loop: IRBlockContainer) -> bool:
        """
        Match ILSpy canonical while transform input:
        entry:
            if (cond) <body-target or body-block>
            leave loop
        """
        entry = loop.entry_block
        if entry is None or len(entry.statements) != 2:
            return False

        if_inst = entry.statements[0]
        exit_inst = entry.statements[1]
        if not isinstance(if_inst, IRIf):
            return False
        if if_inst.false_statement is not None:
            return False
        if not self._is_leave_to_container(exit_inst, loop):
            return False

        true_target = self._as_jump_target(if_inst.true_statement)
        loop_body: Optional[IRBlock] = None

        if true_target is not None:
            if true_target not in loop.blocks:
                return False
            loop_body = true_target
        elif isinstance(if_inst.true_statement, IRBlock):
            loop_body = if_inst.true_statement
            if_inst.true_statement = IRJump(target=loop_body)
            loop.blocks.insert(1, loop_body)
            if not self._has_unreachable_endpoint(loop_body):
                loop_body.statements.append(IRLeave(target_container=loop))
        else:
            return False

        if_inst.false_statement = exit_inst
        entry.statements.pop()
        loop.kind = IRContainerKind.WHILE

        assert loop_body is not None
        while self._unwrap_while_tail_condition(loop_body, loop):
            continue

        return True

    def _unwrap_while_tail_condition(
        self,
        loop_body: IRBlock,
        loop: IRBlockContainer,
    ) -> bool:
        if len(loop_body.statements) < 2:
            return False

        leave_inst = loop_body.statements[-1]
        nested = loop_body.statements[-2]

        if not (
            isinstance(leave_inst, IRLeave)
            and leave_inst.target_container is loop
        ):
            return False
        if not (
            isinstance(nested, IRIf)
            and nested.false_statement is None
        ):
            return False

        if isinstance(nested.true_statement, IRBlock):
            nested_block = nested.true_statement
            loop_body.statements.pop()
            loop_body.statements.extend(nested_block.statements)
        elif self._as_jump_target(nested.true_statement) is not None:
            loop_body.statements[-1] = nested.true_statement
        else:
            return False

        nested.condition = self._logic_not(nested.condition)
        nested.true_statement = leave_inst

        if not self._has_unreachable_endpoint(loop_body):
            loop_body.statements.append(IRLeave(target_container=loop))

        return True

    def _match_do_while_loop(self, loop: IRBlockContainer) -> bool:
        match = self._analyze_do_while_conditions(loop)
        if match is None or not match.conditions:
            return False

        if match.unwrap:
            assert match.unwrap_return_condition is not None
            assert match.unwrap_leave_statement is not None
            assert match.nested_condition_block is not None

            return_if = match.unwrap_return_condition
            leave_function = match.unwrap_leave_statement
            return_if.condition = self._logic_not(return_if.condition)
            return_if.true_statement = leave_function

            if (
                match.original_block.statements
                and match.original_block.statements[-1] is leave_function
            ):
                match.original_block.statements.pop()

            match.original_block.statements.extend(
                match.nested_condition_block.statements
            )
            match.condition_block = match.original_block
            match.split = True

        start = len(match.condition_block.statements) - len(match.conditions) - 1
        if start < 0:
            return False
        del match.condition_block.statements[start:]

        if match.split:
            new_condition_block = IRBlock(
                statements=[],
                start_address=self._next_synthetic_block_address(),
            )
            loop.blocks.append(new_condition_block)
            match.condition_block.statements.append(
                IRJump(target=new_condition_block)
            )
            condition_block = new_condition_block
        else:
            if match.condition_block in loop.blocks:
                loop.blocks.remove(match.condition_block)
                loop.blocks.append(match.condition_block)
            condition_block = match.condition_block

        combined: Optional[IRIf] = None
        for inst in match.conditions:
            if combined is None:
                combined = inst
                if match.swap_branches:
                    combined.condition = self._logic_not(combined.condition)
                    old_true = combined.true_statement
                    combined.false_statement = old_true
                    combined.true_statement = match.exit_statement
                else:
                    combined.false_statement = match.exit_statement
            else:
                if match.swap_branches:
                    combined.condition = self._logic_and(
                        self._logic_not(inst.condition),
                        combined.condition,
                    )
                else:
                    combined.condition = self._logic_and(
                        inst.condition,
                        combined.condition,
                    )

        if combined is None:
            return False

        condition_block.statements.append(combined)
        loop.kind = IRContainerKind.DO_WHILE
        return True

    def _analyze_do_while_conditions(
        self,
        loop: IRBlockContainer,
    ) -> Optional[_DoWhileMatch]:
        for block in reversed(loop.blocks):
            condition_info = self._match_do_while_condition_block(loop, block)
            if condition_info is None:
                continue

            (
                swap_branches,
                unwrap,
                condition_block,
                unwrap_return_if,
                unwrap_leave_statement,
                nested_condition_block,
            ) = condition_info

            conditions = self._collect_conditions(
                loop,
                condition_block,
                swap_branches,
            )
            if not conditions:
                continue

            split = (
                condition_block is loop.entry_block
                or len(condition_block.statements) > len(conditions) + 1
            )

            return _DoWhileMatch(
                conditions=conditions,
                exit_statement=condition_block.statements[-1],
                swap_branches=swap_branches,
                split=split,
                original_block=block,
                condition_block=condition_block,
                unwrap=unwrap,
                unwrap_return_condition=unwrap_return_if,
                unwrap_leave_statement=unwrap_leave_statement,
                nested_condition_block=nested_condition_block,
            )

        return None

    def _match_do_while_condition_block(
        self,
        loop: IRBlockContainer,
        block: IRBlock,
    ) -> Optional[
        tuple[
            bool,
            bool,
            IRBlock,
            Optional[IRIf],
            Optional[IRStatement],
            Optional[IRBlock],
        ]
    ]:
        # Expected endings:
        # 1) if (cond) br entry; leave loop
        # 2) if (cond) leave loop; br entry
        # plus ILSpy's return+nested-condition variant.
        if len(block.statements) < 2:
            return None

        last = block.statements[-1]
        maybe_if = block.statements[-2]
        if not (
            isinstance(maybe_if, IRIf)
            and maybe_if.false_statement is None
        ):
            return None

        condition_block = block
        unwrap = False
        unwrap_return_if: Optional[IRIf] = None
        unwrap_leave_statement: Optional[IRStatement] = None
        nested_condition_block: Optional[IRBlock] = None

        if self._is_function_exit(last) and isinstance(
            maybe_if.true_statement,
            IRBlock,
        ):
            nested = maybe_if.true_statement
            if len(nested.statements) < 2:
                return None
            nested_last = nested.statements[-1]
            nested_if = nested.statements[-2]
            if not (
                isinstance(nested_if, IRIf)
                and nested_if.false_statement is None
            ):
                return None
            unwrap = True
            unwrap_return_if = maybe_if
            unwrap_leave_statement = last
            nested_condition_block = nested
            condition_block = nested
            maybe_if = nested_if
            last = nested_last

        if self._is_jump_to_entry(last, loop):
            swap_branches = True
        elif self._is_leave_to_container(last, loop):
            swap_branches = False
        else:
            return None

        if swap_branches:
            if not self._is_leave_to_container(maybe_if.true_statement, loop):
                return None
        else:
            if not self._is_jump_to_entry(maybe_if.true_statement, loop):
                return None

        return (
            swap_branches,
            unwrap,
            condition_block,
            unwrap_return_if,
            unwrap_leave_statement,
            nested_condition_block,
        )

    def _collect_conditions(
        self,
        loop: IRBlockContainer,
        block: IRBlock,
        swap_branches: bool,
    ) -> list[IRIf]:
        conditions: list[IRIf] = []
        index = len(block.statements) - 2
        while index >= 0:
            statement = block.statements[index]
            if not isinstance(statement, IRIf):
                break
            if statement.false_statement is not None:
                break

            if swap_branches:
                if not self._is_leave_to_container(statement.true_statement, loop):
                    break
            else:
                if not self._is_jump_to_entry(statement.true_statement, loop):
                    break

            conditions.append(statement)
            index -= 1

        return conditions

    def _is_jump_to_entry(
        self,
        statement: IRStatement,
        loop: IRBlockContainer,
    ) -> bool:
        entry = loop.entry_block
        if entry is None:
            return False
        target = self._as_jump_target(statement)
        return target is entry

    def _as_jump_target(self, statement: IRStatement) -> Optional[IRBlock]:
        if isinstance(statement, IRJump):
            return statement.target

        if isinstance(statement, IRBlock) and len(statement.statements) == 1:
            nested = statement.statements[0]
            if isinstance(nested, IRJump):
                return nested.target

        return None

    def _is_leave_to_container(
        self,
        statement: IRStatement,
        container: IRBlockContainer,
    ) -> bool:
        if isinstance(statement, IRLeave):
            return statement.target_container is container

        if isinstance(statement, IRBlock) and len(statement.statements) == 1:
            nested = statement.statements[0]
            return (
                isinstance(nested, IRLeave)
                and nested.target_container is container
            )

        return False

    def _is_function_exit(self, statement: IRStatement) -> bool:
        if isinstance(statement, IRReturn):
            return True
        if isinstance(statement, IRLeave):
            return statement.target_container is self._function_body
        return False

    def _has_unreachable_endpoint(self, block: IRBlock) -> bool:
        if not block.statements:
            return False
        return self._statement_unreachable_endpoint(block.statements[-1])

    def _statement_unreachable_endpoint(self, statement: IRStatement) -> bool:
        if isinstance(statement, (IRJump, IRLeave, IRReturn, IRSwitch)):
            return True

        if isinstance(statement, IRBlock):
            if not statement.statements:
                return False
            return self._statement_unreachable_endpoint(statement.statements[-1])

        if isinstance(statement, IRIf):
            if statement.false_statement is None:
                return False
            return self._statement_unreachable_endpoint(
                statement.true_statement
            ) and self._statement_unreachable_endpoint(statement.false_statement)

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

    def _logic_and(self, left: IRExpression, right: IRExpression) -> IRExpression:
        return IROperatorCallExpression(
            arguments=[left, right],
            operator=Operator.LogicalAnd,
        )

    def _next_synthetic_block_address(self) -> int:
        key = "_synthetic_block_addr"
        current = self._context.metadata.get(key)
        if not isinstance(current, int):
            current = -1
        self._context.metadata[key] = current - 1
        return current
