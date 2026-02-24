from __future__ import annotations

from typing import Optional, cast

from udon_decompiler.analysis.ir.control_flow_node import ControlFlowNode
from udon_decompiler.analysis.ir.nodes import (
    IRBlock,
    IRBlockContainer,
    IRExpression,
    IRFunction,
    IRHighLevelDoWhile,
    IRHighLevelSwitch,
    IRHighLevelWhile,
    IRIf,
    IRJump,
    IRLeave,
    IROperatorCallExpression,
    IRStatement,
    IRSwitch,
)
from udon_decompiler.analysis.operator import Operator
from udon_decompiler.analysis.transform.pass_base import (
    BlockTransformContext,
    IBlockTransform,
)
from udon_decompiler.analysis.transform.passes.detect_exit_points import (
    DetectExitPoints,
)


class ConditionDetection(IBlockTransform):
    def __init__(self) -> None:
        self.context: Optional[BlockTransformContext] = None
        self.current_container: Optional[IRBlockContainer] = None

    def run(self, block: IRBlock, context: BlockTransformContext) -> None:
        if block not in context.container.blocks:
            return

        self.context = context
        self.current_container = context.container

        cfg_node = context.control_flow_node
        if cfg_node is None:
            return
        if cfg_node.block is not block:
            try:
                cfg_node = context.control_flow_graph.get_node(block)
            except KeyError:
                return

        pair = self._get_if_with_exit(block)
        if pair is not None:
            if_inst, exit_inst = pair
            self._handle_if_instruction(cfg_node, block, if_inst, exit_inst)

        if block.statements:
            last = block.statements[-1]
            if self._is_usable_branch_to_child(cfg_node, last):
                self._inline_unconditional_branch(block)

    def _get_if_with_exit(
        self,
        block: IRBlock,
    ) -> Optional[tuple[IRIf, IRStatement]]:
        if len(block.statements) >= 2:
            candidate_if = block.statements[-2]
            exit_inst = block.statements[-1]
            if (
                isinstance(candidate_if, IRIf)
                and candidate_if.false_statement is None
                and self._is_branch_or_leave(exit_inst)
            ):
                return candidate_if, exit_inst

        if not block.statements:
            return None

        terminal_if = block.statements[-1]
        if (
            isinstance(terminal_if, IRIf)
            and terminal_if.false_statement is not None
            and self._is_branch_or_leave(terminal_if.false_statement)
        ):
            exit_inst = terminal_if.false_statement
            terminal_if.false_statement = None
            block.statements.append(exit_inst)
            return terminal_if, exit_inst

        return None

    def _handle_if_instruction(
        self,
        cfg_node: ControlFlowNode,
        block: IRBlock,
        if_inst: IRIf,
        exit_inst: IRStatement,
    ) -> None:
        if self._should_swap_if_targets(if_inst.true_statement, exit_inst):
            if_inst.true_statement, block.statements[-1] = (
                block.statements[-1],
                if_inst.true_statement,
            )
            exit_inst = block.statements[-1]
            if_inst.condition = self._logic_not(if_inst.condition)

        true_exit_inst: Optional[IRStatement]
        if self._is_usable_branch_to_child(cfg_node, if_inst.true_statement):
            target_block = cast(IRJump, if_inst.true_statement).target
            self._remove_block_from_current_container(target_block)
            if_inst.true_statement = target_block

            while (
                target_block.statements
                and isinstance(target_block.statements[0], IRIf)
                and target_block.statements[0].false_statement is None
            ):
                nested_if = cast(IRIf, target_block.statements[0])
                nested_true = self._unpack_block_containing_only_branch(
                    nested_if.true_statement
                )
                if DetectExitPoints.compatible_exit_instruction(exit_inst, nested_true):
                    if_inst.condition = self._logic_and(
                        if_inst.condition,
                        self._logic_not(nested_if.condition),
                    )
                    target_block.statements.pop(0)
                    continue

                if (
                    isinstance(nested_true, IRBlock)
                    and nested_true.statements
                    and len(target_block.statements) > 1
                    and self._contains_non_exit_statement(target_block.statements[1:])
                    and DetectExitPoints.compatible_exit_instruction(
                        exit_inst,
                        nested_true.statements[-1],
                    )
                    and self._has_unreachable_endpoint(target_block)
                ):
                    # if (...) { if (nested) { true...; goto exit; } false...; }
                    # goto exit;
                    # -> if (...) { if (!nested) { false...; } true...; } goto exit;
                    nested_if.condition = self._logic_not(nested_if.condition)
                    nested_true.statements.pop()

                    false_statements = target_block.statements[1:]
                    target_block.statements = target_block.statements[:1]
                    target_block.statements.extend(nested_true.statements)
                    nested_true.statements = false_statements
                break

            true_exit_inst = self._last_exit_statement(target_block)
            if DetectExitPoints.compatible_exit_instruction(exit_inst, true_exit_inst):
                assert true_exit_inst is not None
                target_block.statements.pop()
                true_exit_inst = None

                if (
                    len(target_block.statements) == 1
                    and isinstance(target_block.statements[0], IRIf)
                    and target_block.statements[0].false_statement is None
                ):
                    nested_if = cast(IRIf, target_block.statements[0])
                    if_inst.condition = self._logic_and(
                        if_inst.condition,
                        nested_if.condition,
                    )
                    if_inst.true_statement = nested_if.true_statement
                    if isinstance(if_inst.true_statement, IRBlock):
                        true_exit_inst = self._last_exit_statement(
                            if_inst.true_statement
                        )
                    elif self._is_branch_or_leave(if_inst.true_statement):
                        true_exit_inst = if_inst.true_statement
                    else:
                        true_exit_inst = None
        else:
            if self._is_branch_or_leave(if_inst.true_statement):
                true_exit_inst = if_inst.true_statement
            else:
                true_exit_inst = None

        if self._is_usable_branch_to_child(cfg_node, exit_inst):
            target_block = cast(IRJump, exit_inst).target
            false_exit_inst = self._last_exit_statement(target_block)
            if DetectExitPoints.compatible_exit_instruction(
                true_exit_inst,
                false_exit_inst,
            ):
                assert false_exit_inst is not None
                target_block.statements.pop()
                self._remove_block_from_current_container(target_block)
                if_inst.false_statement = (
                    target_block if target_block.statements else None
                )

                block.statements[-1] = false_exit_inst
                exit_inst = false_exit_inst

                true_block = (
                    if_inst.true_statement
                    if isinstance(if_inst.true_statement, IRBlock)
                    else None
                )
                if true_block is not None:
                    if (
                        true_block.statements
                        and true_block.statements[-1] is true_exit_inst
                    ):
                        true_block.statements.pop()
                elif true_exit_inst is if_inst.true_statement:
                    if_inst.true_statement = self._empty_block()

        if self._is_empty(if_inst.true_statement):
            old_true = if_inst.true_statement
            if_inst.true_statement = (
                if_inst.false_statement
                if if_inst.false_statement is not None
                else self._empty_block()
            )
            if_inst.false_statement = old_true
            if_inst.condition = self._logic_not(if_inst.condition)

            true_block = (
                if_inst.true_statement
                if isinstance(if_inst.true_statement, IRBlock)
                else None
            )
            if (
                true_block is not None
                and len(true_block.statements) == 1
                and isinstance(true_block.statements[0], IRIf)
                and true_block.statements[0].false_statement is None
            ):
                nested_if = cast(IRIf, true_block.statements[0])
                if_inst.condition = self._logic_and(
                    if_inst.condition,
                    nested_if.condition,
                )
                if_inst.true_statement = nested_if.true_statement
        elif if_inst.false_statement is not None and self._statement_start(
            if_inst.false_statement
        ) < self._statement_start(if_inst.true_statement):
            old_true = if_inst.true_statement
            if_inst.true_statement = if_inst.false_statement
            if_inst.false_statement = old_true
            if_inst.condition = self._logic_not(if_inst.condition)

        _ = exit_inst

    def _inline_unconditional_branch(self, block: IRBlock) -> None:
        last = block.statements[-1]
        if not isinstance(last, IRJump):
            return

        target_block = last.target
        if target_block is block:
            return
        self._remove_block_from_current_container(target_block)

        block.statements.pop()
        block.statements.extend(target_block.statements)

    def _should_swap_if_targets(self, inst1: IRStatement, inst2: IRStatement) -> bool:
        if isinstance(inst1, IRJump) and isinstance(inst2, IRJump):
            return inst1.target.start_address > inst2.target.start_address

        if isinstance(inst1, IRLeave) and not isinstance(inst2, IRLeave):
            return True

        if isinstance(inst1, IRJump) and isinstance(inst2, IRLeave):
            if self._incoming_edge_count(inst1.target) > 1:
                return True

        return False

    def _is_usable_branch_to_child(
        self,
        cfg_node: ControlFlowNode,
        potential_branch_instruction: IRStatement,
    ) -> bool:
        if not isinstance(potential_branch_instruction, IRJump):
            return False

        if self.current_container is None:
            return False

        target_block = potential_branch_instruction.target
        if target_block is cfg_node.block:
            return False
        if target_block not in self.current_container.blocks:
            return False

        if self._incoming_edge_count(target_block) != 1:
            return False

        assert self.context is not None
        try:
            target_node = self.context.control_flow_graph.get_node(target_block)
        except KeyError:
            return False

        if not cfg_node.dominates(target_node):
            return False

        return True

    def _incoming_edge_count(self, block: IRBlock) -> int:
        function = self._current_function()
        count = 0
        for source in function.body.blocks:
            count += self._count_targets_in_statement_list(
                source.statements,
                block,
            )
        return count

    def _count_targets_in_statement_list(
        self,
        statements: list[IRStatement],
        target: IRBlock,
    ) -> int:
        count = 0
        for statement in statements:
            count += self._count_targets_in_statement(statement, target)
        return count

    def _count_targets_in_statement(
        self,
        statement: IRStatement,
        target: IRBlock,
    ) -> int:
        if isinstance(statement, IRJump):
            return 1 if statement.target is target else 0

        if isinstance(statement, IRIf):
            count = self._count_targets_in_statement(statement.true_statement, target)
            if statement.false_statement is not None:
                count += self._count_targets_in_statement(
                    statement.false_statement,
                    target,
                )
            return count

        if isinstance(statement, IRBlock):
            return self._count_targets_in_statement_list(
                statement.statements,
                target,
            )

        if isinstance(statement, IRBlockContainer):
            total = 0
            for block in statement.blocks:
                total += self._count_targets_in_statement_list(block.statements, target)
            return total

        if isinstance(statement, IRHighLevelSwitch):
            total = 0
            for section in statement.sections:
                total += self._count_targets_in_statement(
                    section.body,
                    target,
                )
            return total

        if isinstance(statement, (IRHighLevelWhile, IRHighLevelDoWhile)):
            total = 0
            for block in statement.body.blocks:
                total += self._count_targets_in_statement_list(
                    block.statements,
                    target,
                )
            return total

        if isinstance(statement, IRSwitch):
            total = 0
            for case_target in statement.cases.values():
                if case_target is target:
                    total += 1
            if statement.default_target is target:
                total += 1
            return total

        return 0

    @staticmethod
    def _has_unreachable_endpoint(block: IRBlock) -> bool:
        if not block.statements:
            return False
        return isinstance(block.statements[-1], (IRJump, IRLeave))

    def _contains_non_exit_statement(
        self,
        statements: list[IRStatement],
    ) -> bool:
        for statement in statements:
            if not self._is_branch_or_leave(statement):
                return True
        return False

    def _current_function(self) -> IRFunction:
        assert self.context is not None
        return self.context.function

    def _remove_block_from_current_container(self, block: IRBlock) -> None:
        if self.current_container is None:
            return
        if block in self.current_container.blocks:
            self.current_container.blocks.remove(block)

    @staticmethod
    def _last_exit_statement(block: IRBlock) -> Optional[IRStatement]:
        if not block.statements:
            return None
        last = block.statements[-1]
        if isinstance(last, (IRJump, IRLeave)):
            return last
        return None

    def _unpack_block_containing_only_branch(
        self,
        statement: IRStatement,
    ) -> IRStatement:
        if (
            isinstance(statement, IRBlock)
            and len(statement.statements) == 1
            and self._is_branch_or_leave(statement.statements[0])
        ):
            return statement.statements[0]
        return statement

    @staticmethod
    def _is_branch_or_leave(statement: object) -> bool:
        return isinstance(statement, (IRJump, IRLeave))

    @staticmethod
    def _is_empty(statement: IRStatement) -> bool:
        return isinstance(statement, IRBlock) and len(statement.statements) == 0

    def _empty_block(self) -> IRBlock:
        return IRBlock(
            statements=[],
            start_address=self._next_synthetic_block_address(),
        )

    def _next_synthetic_block_address(self) -> int:
        assert self.context is not None

        key = "_synthetic_block_addr"
        current = self.context.metadata.get(key)
        if not isinstance(current, int):
            current = -1
        self.context.metadata[key] = current - 1
        return current

    @staticmethod
    def _statement_start(statement: IRStatement) -> int:
        if isinstance(statement, IRBlock):
            return statement.start_address
        return 2**31 - 1

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

    @staticmethod
    def _logic_and(lhs: IRExpression, rhs: IRExpression) -> IRExpression:
        return IROperatorCallExpression(
            arguments=[lhs, rhs],
            operator=Operator.LogicalAnd,
        )
