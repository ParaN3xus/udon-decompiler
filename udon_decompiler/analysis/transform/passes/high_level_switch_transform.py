from __future__ import annotations

from copy import deepcopy
from typing import Dict, List, Optional, cast

from udon_decompiler.analysis.ir.nodes import (
    IRBlock,
    IRBlockContainer,
    IRContainerKind,
    IRFunction,
    IRHighLevelSwitch,
    IRHighLevelSwitchSection,
    IRIf,
    IRJump,
    IRLeave,
    IRReturn,
    IRStatement,
    IRSwitch,
)
from udon_decompiler.analysis.transform.pass_base import (
    IILTransform,
    ILTransformContext,
)


class HighLevelSwitchTransform(IILTransform):
    """
    Lift low-level switch containers into high-level switch sections.

    The conversion follows ILSpy StatementBuilder's core ideas:
    - group labels by target block
    - pick a default section (largest label group when there is no explicit default)
    - inline section bodies when the target block is switch-private
    """

    def run(self, function: IRFunction, context: ILTransformContext) -> None:
        _ = context
        self._rewrite_container(function.body)

    def _rewrite_container(self, container: IRBlockContainer) -> None:
        for block in list(container.blocks):
            block.statements = self._rewrite_statement_list(
                block.statements,
                parent_container=container,
                parent_block=block,
            )

    def _rewrite_statement_list(
        self,
        statements: List[IRStatement],
        parent_container: IRBlockContainer,
        parent_block: IRBlock,
    ) -> List[IRStatement]:
        return [
            self._rewrite_statement(
                statement=statement,
                parent_container=parent_container,
                parent_block=parent_block,
            )
            for statement in statements
        ]

    def _rewrite_statement(
        self,
        statement: IRStatement,
        parent_container: IRBlockContainer,
        parent_block: IRBlock,
    ) -> IRStatement:
        if isinstance(statement, IRIf):
            statement.true_statement = self._rewrite_statement(
                statement.true_statement,
                parent_container=parent_container,
                parent_block=parent_block,
            )
            if statement.false_statement is not None:
                statement.false_statement = self._rewrite_statement(
                    statement.false_statement,
                    parent_container=parent_container,
                    parent_block=parent_block,
                )
            return statement

        if isinstance(statement, IRBlock):
            statement.statements = self._rewrite_statement_list(
                statement.statements,
                parent_container=parent_container,
                parent_block=statement,
            )
            return statement

        if isinstance(statement, IRBlockContainer):
            self._rewrite_container(statement)
            if statement.kind == IRContainerKind.SWITCH:
                return self._lift_switch_container(
                    statement,
                    parent_container,
                    parent_block,
                )
            return statement

        return statement

    def _lift_switch_container(
        self,
        switch_container: IRBlockContainer,
        parent_container: IRBlockContainer,
        parent_block: IRBlock,
    ) -> IRStatement:
        entry = switch_container.entry_block
        if entry is None or len(entry.statements) != 1:
            return switch_container

        switch_inst = entry.statements[0]
        if not isinstance(switch_inst, IRSwitch):
            return switch_container

        target_to_labels: Dict[IRBlock, List[int]] = {}
        for label_value, target in switch_inst.cases.items():
            target_to_labels.setdefault(target, []).append(label_value)
        if not target_to_labels:
            return switch_container

        explicit_default_target = switch_inst.default_target
        default_target = explicit_default_target
        largest_case_target: Optional[IRBlock] = None

        if default_target is None:
            largest_case_target = max(
                target_to_labels.items(),
                key=lambda item: len(item[1]),
            )[0]
            default_target = largest_case_target

        unique_targets: List[IRBlock] = []
        for target in target_to_labels:
            if target not in unique_targets:
                unique_targets.append(target)
        if default_target is not None and default_target not in unique_targets:
            unique_targets.append(default_target)

        incoming_counts = self._count_incoming_edges(parent_container)
        switch_ref_counts = self._count_switch_refs(switch_inst)
        common_exit = self._detect_common_exit(unique_targets)
        can_drop_common_exit_jump = parent_block in parent_container.blocks

        sections: List[IRHighLevelSwitchSection] = []
        consumed_blocks: List[IRBlock] = []

        non_default_targets = [t for t in unique_targets if t is not default_target]
        non_default_targets.sort(
            key=lambda target: min(target_to_labels.get(target, [2**31 - 1]))
        )

        for target in non_default_targets:
            labels = sorted(target_to_labels.get(target, []))
            body = self._build_section_body(
                target=target,
                switch_container=switch_container,
                parent_container=parent_container,
                incoming_counts=incoming_counts,
                switch_ref_counts=switch_ref_counts,
                common_exit=common_exit,
                can_drop_common_exit_jump=can_drop_common_exit_jump,
            )
            if (
                body.start_address == target.start_address
                and target in parent_container.blocks
            ):
                if self._is_consumable_target(
                    target=target,
                    parent_container=parent_container,
                    incoming_counts=incoming_counts,
                    switch_ref_counts=switch_ref_counts,
                    switch_container=switch_container,
                    common_exit=common_exit,
                ):
                    consumed_blocks.append(target)
            sections.append(
                IRHighLevelSwitchSection(
                    labels=labels,
                    body=body,
                    is_default=False,
                )
            )

        if default_target is not None:
            default_labels = sorted(target_to_labels.get(default_target, []))
            if (
                largest_case_target is default_target
                and explicit_default_target is None
            ):
                default_labels = []

            default_body = self._build_section_body(
                target=default_target,
                switch_container=switch_container,
                parent_container=parent_container,
                incoming_counts=incoming_counts,
                switch_ref_counts=switch_ref_counts,
                common_exit=common_exit,
                can_drop_common_exit_jump=can_drop_common_exit_jump,
            )
            if (
                default_body.start_address == default_target.start_address
                and default_target in parent_container.blocks
                and self._is_consumable_target(
                    target=default_target,
                    parent_container=parent_container,
                    incoming_counts=incoming_counts,
                    switch_ref_counts=switch_ref_counts,
                    switch_container=switch_container,
                    common_exit=common_exit,
                )
            ):
                consumed_blocks.append(default_target)

            sections.append(
                IRHighLevelSwitchSection(
                    labels=default_labels,
                    body=default_body,
                    is_default=True,
                )
            )

        for block in consumed_blocks:
            if block in parent_container.blocks:
                parent_container.blocks.remove(block)

        return IRHighLevelSwitch(
            index_expression=switch_inst.index_expression,
            sections=sections,
        )

    def _build_section_body(
        self,
        target: IRBlock,
        switch_container: IRBlockContainer,
        parent_container: IRBlockContainer,
        incoming_counts: Dict[IRBlock, int],
        switch_ref_counts: Dict[IRBlock, int],
        common_exit: Optional[IRBlock],
        can_drop_common_exit_jump: bool,
    ) -> IRBlock:
        # Most lowered switches use internal switch-container target blocks
        # ending with `leave switch`. These blocks must be inlined into
        # section bodies; emitting `jump -> target` would become dangling
        # once the switch container gets replaced by IRHighLevelSwitch.
        if target in switch_container.blocks:
            copied_statements: List[IRStatement] = list(target.statements)
            if copied_statements:
                tail = copied_statements[-1]
                if (
                    isinstance(tail, IRLeave)
                    and tail.target_container is switch_container
                ):
                    copied_statements.pop()
                elif (
                    isinstance(tail, IRJump)
                    and common_exit is not None
                    and tail.target is common_exit
                    and common_exit not in parent_container.blocks
                ):
                    copied_statements.pop()
            return IRBlock(
                statements=copied_statements,
                start_address=target.start_address,
            )

        consumable = self._is_consumable_target(
            target=target,
            parent_container=parent_container,
            incoming_counts=incoming_counts,
            switch_ref_counts=switch_ref_counts,
            switch_container=switch_container,
            common_exit=common_exit,
        )

        if not consumable:
            # Pattern seen in range-guarded lowered switches:
            # target block was inlined by an earlier pass, but we still have the
            # original target block object and can inline its payload safely.
            if (
                target not in parent_container.blocks
                and target.statements
                and isinstance(target.statements[-1], IRJump)
                and common_exit is not None
                and cast(IRJump, target.statements[-1]).target is common_exit
            ):
                copied = deepcopy(target.statements)
                # Keep the trailing jump when its destination is still emitted in
                # this container. This preserves "assign then goto exit" shape and
                # avoids duplicated fall-through assignments after the switch.
                if common_exit not in parent_container.blocks:
                    copied = copied[:-1]
                return IRBlock(
                    statements=copied,
                    start_address=target.start_address,
                )
            return IRBlock(
                statements=[IRJump(target=target)],
                start_address=target.start_address,
            )

        drop_last_jump = False
        drop_last_leave = False
        if target.statements:
            original_last = target.statements[-1]
            if (
                isinstance(original_last, IRJump)
                and can_drop_common_exit_jump
                and common_exit is not None
                and original_last.target is common_exit
            ):
                drop_last_jump = True
            elif (
                isinstance(original_last, IRLeave)
                and original_last.target_container is switch_container
            ):
                drop_last_leave = True

        # Preserve block target identity in IRJump so later label collection can
        # mark the real destination block.
        copied_statements: List[IRStatement] = list(target.statements)
        if copied_statements and (drop_last_jump or drop_last_leave):
            copied_statements.pop()

        return IRBlock(
            statements=copied_statements,
            start_address=target.start_address,
        )

    def _is_consumable_target(
        self,
        target: IRBlock,
        parent_container: IRBlockContainer,
        incoming_counts: Dict[IRBlock, int],
        switch_ref_counts: Dict[IRBlock, int],
        switch_container: IRBlockContainer,
        common_exit: Optional[IRBlock],
    ) -> bool:
        if target not in parent_container.blocks:
            return False

        if incoming_counts.get(target, 0) != switch_ref_counts.get(target, 0):
            return False

        if not target.statements:
            return True

        last = target.statements[-1]
        if isinstance(last, IRJump):
            return common_exit is not None and last.target is common_exit
        if isinstance(last, IRLeave):
            return last.target_container is switch_container
        return isinstance(last, IRReturn)

    def _detect_common_exit(self, targets: List[IRBlock]) -> Optional[IRBlock]:
        target_set = set(targets)
        counts: Dict[IRBlock, int] = {}
        for target in targets:
            if not target.statements:
                continue
            last = target.statements[-1]
            if isinstance(last, IRJump):
                exit_target = last.target
                if exit_target in target_set:
                    continue
                counts[exit_target] = counts.get(exit_target, 0) + 1

        if not counts:
            return None

        best_target, best_count = max(counts.items(), key=lambda item: item[1])
        if best_count < 2:
            return None
        return best_target

    def _count_switch_refs(self, switch_inst: IRSwitch) -> Dict[IRBlock, int]:
        counts: Dict[IRBlock, int] = {}
        for target in switch_inst.cases.values():
            counts[target] = counts.get(target, 0) + 1
        if switch_inst.default_target is not None:
            default_target = switch_inst.default_target
            counts[default_target] = counts.get(default_target, 0) + 1
        return counts

    def _count_incoming_edges(self, container: IRBlockContainer) -> Dict[IRBlock, int]:
        counts: Dict[IRBlock, int] = {}
        for block in container.blocks:
            for statement in block.statements:
                self._count_targets_in_statement(statement, counts)
        return counts

    def _count_targets_in_statement(
        self,
        statement: IRStatement,
        counts: Dict[IRBlock, int],
    ) -> None:
        if isinstance(statement, IRJump):
            counts[statement.target] = counts.get(statement.target, 0) + 1
            return

        if isinstance(statement, IRIf):
            self._count_targets_in_statement(statement.true_statement, counts)
            if statement.false_statement is not None:
                self._count_targets_in_statement(statement.false_statement, counts)
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
                self._count_targets_in_statement(section.body, counts)
            return

        if isinstance(statement, IRBlock):
            for nested in statement.statements:
                self._count_targets_in_statement(nested, counts)
            return

        if isinstance(statement, IRBlockContainer):
            for block in statement.blocks:
                for nested in block.statements:
                    self._count_targets_in_statement(nested, counts)
