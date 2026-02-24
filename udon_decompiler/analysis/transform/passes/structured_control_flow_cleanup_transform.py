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
    IRReturn,
    IRStatement,
    IRSwitch,
)
from udon_decompiler.analysis.transform.pass_base import (
    ITransform,
    TransformContext,
)


class StructuredControlFlowCleanupTransform(ITransform):
    """
    Cleanup pass for structured control flow.

    Current scope:
    - remove truly-empty else-branches (`else {}`) represented as an empty IRBlock.
    """

    def run(self, function: IRFunction, context: TransformContext) -> None:
        self._context = context
        self._root_body = function.body
        self._rewrite_container(function.body)

    def _rewrite_container(self, container: IRBlockContainer) -> None:
        for block in container.blocks:
            block.statements = [
                self._rewrite_statement(stmt) for stmt in block.statements
            ]
            self._simplify_if_else_join_jumps(block)
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
            self._simplify_if_else_join_jumps(statement)
            self._simplify_if_missing_jump_to_else(statement)
            return statement

        if isinstance(statement, IRBlockContainer):
            self._rewrite_container(statement)
            return statement

        if isinstance(statement, IRHighLevelWhile):
            statement.body = cast(
                IRBlockContainer,
                self._rewrite_statement(statement.body),
            )
            self._eliminate_nested_gotos_to_next_block(statement.body)
            self._simplify_linear_if_goto_diamonds(statement.body)
            self._simplify_two_way_if_goto_diamonds(statement.body)
            self._simplify_true_terminal_if_gotos(statement.body)
            self._simplify_terminal_two_way_if_gotos(statement.body)
            self._simplify_guarded_linear_regions(statement.body)
            self._simplify_late_true_target_if_gotos(statement.body)
            self._simplify_nested_if_goto_fallbacks(statement.body)
            return statement

        if isinstance(statement, IRHighLevelDoWhile):
            statement.body = cast(
                IRBlockContainer,
                self._rewrite_statement(statement.body),
            )
            self._eliminate_nested_gotos_to_next_block(statement.body)
            self._simplify_linear_if_goto_diamonds(statement.body)
            self._simplify_two_way_if_goto_diamonds(statement.body)
            self._simplify_true_terminal_if_gotos(statement.body)
            self._simplify_terminal_two_way_if_gotos(statement.body)
            self._simplify_guarded_linear_regions(statement.body)
            self._simplify_late_true_target_if_gotos(statement.body)
            self._simplify_nested_if_goto_fallbacks(statement.body)
            return statement

        if isinstance(statement, IRHighLevelSwitch):
            for section in statement.sections:
                section.body = cast(IRBlock, self._rewrite_statement(section.body))
            return statement

        return statement

    def _eliminate_nested_gotos_to_next_block(
        self,
        body: IRBlockContainer,
    ) -> None:
        """
        Eliminate nested `goto nextBlock` patterns inside loop bodies.

        Typical shape:
            if (outer) {
                ...;
                if (inner) { ...; goto NEXT; }
                tail...; <terminal>;
            }
            goto NEXT;

        Rewritten to:
            if (outer) {
                ...;
                if (inner) { ...; }
                else { tail...; <terminal>; }
            }
            goto NEXT;
        """
        changed = True
        while changed:
            changed = False
            for index, block in enumerate(list(body.blocks)):
                if index + 1 >= len(body.blocks):
                    continue
                next_block = body.blocks[index + 1]
                if self._rewrite_nested_goto_in_statement_list(
                    block.statements,
                    next_block,
                ):
                    changed = True
                    break

    def _rewrite_nested_goto_in_statement_list(
        self,
        statements: list[IRStatement],
        next_block: IRBlock,
    ) -> bool:
        changed = True
        any_changed = False
        while changed:
            changed = False
            for index, statement in enumerate(list(statements)):
                if self._rewrite_nested_goto_in_statement(
                    statement,
                    next_block,
                ):
                    changed = True
                    any_changed = True
                    break

                if not isinstance(statement, IRIf):
                    continue
                if statement.false_statement is not None:
                    continue
                if not isinstance(statement.true_statement, IRBlock):
                    continue

                true_block = statement.true_statement
                if not true_block.statements:
                    continue
                tail = true_block.statements[-1]
                if not isinstance(tail, IRJump) or tail.target is not next_block:
                    continue

                suffix = statements[index + 1 :]
                if not suffix:
                    continue
                if not self._statements_end_unreachable(suffix):
                    continue
                if self._has_nested_control_flow_statement(suffix):
                    continue

                true_block.statements.pop()
                statement.false_statement = IRBlock(
                    statements=list(suffix),
                    start_address=self._new_block_start_address(suffix[0]),
                )
                del statements[index + 1 :]

                changed = True
                any_changed = True
                break

        return any_changed

    def _rewrite_nested_goto_in_statement(
        self,
        statement: IRStatement,
        next_block: IRBlock,
    ) -> bool:
        if isinstance(statement, IRIf):
            changed = self._rewrite_nested_goto_in_statement(
                statement.true_statement,
                next_block,
            )
            if statement.false_statement is not None:
                changed = (
                    self._rewrite_nested_goto_in_statement(
                        statement.false_statement,
                        next_block,
                    )
                    or changed
                )
            return changed

        if isinstance(statement, IRBlock):
            return self._rewrite_nested_goto_in_statement_list(
                statement.statements,
                next_block,
            )

        return False

    def _has_nested_control_flow_statement(
        self,
        statements: list[IRStatement],
    ) -> bool:
        for statement in statements:
            if isinstance(
                statement,
                (
                    IRIf,
                    IRBlockContainer,
                    IRHighLevelSwitch,
                    IRHighLevelWhile,
                    IRHighLevelDoWhile,
                    IRSwitch,
                ),
            ):
                return True
            if isinstance(statement, IRBlock):
                if self._has_nested_control_flow_statement(statement.statements):
                    return True
        return False

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

    def _simplify_two_way_if_goto_diamonds(
        self,
        body: IRBlockContainer,
    ) -> None:
        """
        Simplify shape:
            A: if (c) goto T; goto F;
            T: ...; goto U;
            F: ...; goto U;
            U: ...
        into:
            A: if (c) { ... } else { ... }
            U: ...
        """
        changed = True
        while changed:
            changed = False
            incoming = self._compute_incoming_counts(body)

            for block in list(body.blocks):
                if len(block.statements) < 2:
                    continue

                if_inst = block.statements[-2]
                fallback_jump = block.statements[-1]
                if not isinstance(if_inst, IRIf) or if_inst.false_statement is not None:
                    continue
                if not isinstance(fallback_jump, IRJump):
                    continue

                true_target = self._as_jump_target(if_inst.true_statement)
                false_target = fallback_jump.target
                if true_target is None:
                    continue
                if true_target is false_target:
                    continue
                if true_target not in body.blocks or false_target not in body.blocks:
                    continue
                if (
                    incoming.get(true_target, 0) != 1
                    or incoming.get(false_target, 0) != 1
                ):
                    continue

                true_tail = (
                    true_target.statements[-1] if true_target.statements else None
                )
                false_tail = (
                    false_target.statements[-1] if false_target.statements else None
                )
                if not isinstance(true_tail, IRJump) or not isinstance(
                    false_tail, IRJump
                ):
                    continue
                if true_tail.target is not false_tail.target:
                    continue
                join = true_tail.target
                if join not in body.blocks:
                    continue

                true_statements = list(true_target.statements[:-1])
                false_statements = list(false_target.statements[:-1])

                if_inst.true_statement = IRBlock(
                    statements=true_statements,
                    start_address=true_target.start_address,
                )
                if_inst.false_statement = IRBlock(
                    statements=false_statements,
                    start_address=false_target.start_address,
                )
                block.statements.pop()  # remove trailing goto F

                if true_target in body.blocks:
                    body.blocks.remove(true_target)
                if false_target in body.blocks:
                    body.blocks.remove(false_target)

                changed = True
                break

    def _simplify_true_terminal_if_gotos(
        self,
        body: IRBlockContainer,
    ) -> None:
        """
        Simplify shape:
            A: if (c) goto T; goto F;
            T: ...; <terminal>;
            F: ...
        into:
            A: if (c) { ...; <terminal>; }
            F: ...

        Preconditions keep this conservative:
        - linearized as [A, T, F]
        - T has one incoming edge
        - T endpoint is unreachable (continue/break/leave/return/switch)
        """
        changed = True
        while changed:
            changed = False
            incoming = self._compute_incoming_counts(body)

            for block_index, block in enumerate(list(body.blocks)):
                if block_index >= len(body.blocks):
                    break
                if len(block.statements) < 2:
                    continue

                if_stmt = block.statements[-2]
                fallback_jump = block.statements[-1]
                if not isinstance(if_stmt, IRIf) or if_stmt.false_statement is not None:
                    continue
                if not isinstance(fallback_jump, IRJump):
                    continue

                true_target = self._as_jump_target(if_stmt.true_statement)
                false_target = fallback_jump.target
                if true_target is None:
                    continue
                if true_target not in body.blocks or false_target not in body.blocks:
                    continue
                if incoming.get(true_target, 0) != 1:
                    continue

                if block_index + 2 >= len(body.blocks):
                    continue
                if body.blocks[block_index + 1] is not true_target:
                    continue
                if body.blocks[block_index + 2] is not false_target:
                    continue
                if not self._statements_end_unreachable(true_target.statements):
                    continue

                if_stmt.true_statement = IRBlock(
                    statements=list(true_target.statements),
                    start_address=true_target.start_address,
                )
                block.statements.pop()  # remove goto F (now fallthrough)

                if true_target in body.blocks:
                    body.blocks.remove(true_target)

                changed = True
                break

    def _simplify_terminal_two_way_if_gotos(
        self,
        body: IRBlockContainer,
    ) -> None:
        """
        Simplify shape:
            A: if (c) goto T; goto F;
            T: ...; <terminal>;
            F: ...; <terminal>;
        into:
            A: if (c) { ...; <terminal>; } else { ...; <terminal>; }
        """
        changed = True
        while changed:
            changed = False
            incoming = self._compute_incoming_counts(body)

            for block in list(body.blocks):
                if len(block.statements) < 2:
                    continue

                if_stmt = block.statements[-2]
                fallback_jump = block.statements[-1]
                if not isinstance(if_stmt, IRIf) or if_stmt.false_statement is not None:
                    continue
                if not isinstance(fallback_jump, IRJump):
                    continue

                true_target = self._as_jump_target(if_stmt.true_statement)
                false_target = fallback_jump.target
                if true_target is None:
                    continue
                if true_target is false_target:
                    continue
                if true_target not in body.blocks or false_target not in body.blocks:
                    continue
                if incoming.get(true_target, 0) != 1:
                    continue
                if incoming.get(false_target, 0) != 1:
                    continue

                if not self._statements_end_unreachable(true_target.statements):
                    continue
                if not self._statements_end_unreachable(false_target.statements):
                    continue

                if_stmt.true_statement = IRBlock(
                    statements=list(true_target.statements),
                    start_address=true_target.start_address,
                )
                if_stmt.false_statement = IRBlock(
                    statements=list(false_target.statements),
                    start_address=false_target.start_address,
                )
                block.statements.pop()  # remove trailing goto F

                if true_target in body.blocks:
                    body.blocks.remove(true_target)
                if false_target in body.blocks:
                    body.blocks.remove(false_target)

                changed = True
                break

    def _simplify_guarded_linear_regions(
        self,
        body: IRBlockContainer,
    ) -> None:
        """
        Simplify guarded linear region:
            A: if (c) goto B; goto Z;
            B: ...
            ...
            Y: ...
            Z: ...

        where [B..Y] is a pure linear fallthrough slice only entered from A.
        Rewrite:
            A: if (c) { B...Y... }
            Z: ...
        """
        changed = True
        while changed:
            changed = False
            incoming = self._compute_incoming_counts(body)

            for block_index, block in enumerate(list(body.blocks)):
                if block_index >= len(body.blocks):
                    break
                if len(block.statements) < 2:
                    continue

                if_stmt = block.statements[-2]
                fallback_jump = block.statements[-1]
                if not isinstance(if_stmt, IRIf) or if_stmt.false_statement is not None:
                    continue
                if not isinstance(fallback_jump, IRJump):
                    continue

                region_start = self._as_jump_target(if_stmt.true_statement)
                region_end = fallback_jump.target
                if region_start is None:
                    continue
                if region_start not in body.blocks or region_end not in body.blocks:
                    continue

                start_index = body.blocks.index(region_start)
                end_index = body.blocks.index(region_end)
                if start_index != block_index + 1:
                    continue
                if end_index <= start_index:
                    continue

                region_blocks = body.blocks[start_index:end_index]
                if not region_blocks:
                    continue

                if incoming.get(region_blocks[0], 0) != 1:
                    continue

                if not self._region_has_no_extra_explicit_entries(
                    region_blocks=region_blocks,
                    incoming=incoming,
                ):
                    continue

                if not self._region_is_mergeable(
                    region_blocks=region_blocks,
                    region_end=region_end,
                ):
                    continue

                merged_statements: list[IRStatement] = []
                for region_block in region_blocks:
                    merged_statements.extend(region_block.statements)
                if (
                    merged_statements
                    and isinstance(merged_statements[-1], IRJump)
                    and merged_statements[-1].target is region_end
                ):
                    merged_statements.pop()

                if_stmt.true_statement = IRBlock(
                    statements=merged_statements,
                    start_address=region_blocks[0].start_address,
                )
                block.statements.pop()  # remove goto Z (false now falls through to Z)

                for region_block in region_blocks:
                    if region_block in body.blocks:
                        body.blocks.remove(region_block)

                changed = True
                break

    def _simplify_if_else_join_jumps(self, block: IRBlock) -> None:
        """
        Normalize nested pattern:
            if (c) { ...; goto F; } else { F... }
        into:
            if (c) { ... }
            F...

        This removes dangling gotos when `F` is no longer a physical container block.
        """
        changed = True
        while changed:
            changed = False
            for index, statement in enumerate(list(block.statements)):
                if not isinstance(statement, IRIf):
                    continue
                if not isinstance(statement.true_statement, IRBlock):
                    continue
                if not isinstance(statement.false_statement, IRBlock):
                    continue

                true_block = statement.true_statement
                false_block = statement.false_statement
                ref_count = self._count_jump_target_references_in_statement(
                    statement=true_block,
                    target=false_block,
                )
                if ref_count != 1:
                    continue

                total_ref_count = self._count_jump_target_references_in_block(
                    block=block,
                    target=false_block,
                )
                if total_ref_count != 1:
                    continue

                if not self._remove_first_jump_target_reference(
                    statement=true_block,
                    target=false_block,
                ):
                    continue

                statement.false_statement = None
                block.statements = (
                    block.statements[: index + 1]
                    + list(false_block.statements)
                    + block.statements[index + 1 :]
                )

                changed = True
                break

    def _simplify_if_missing_jump_to_else(self, block: IRBlock) -> None:
        """
        Repair pattern produced by earlier block transforms:
            ...;
            if (c) { ...; goto <missing-target>; }
            <suffix>

        where `<missing-target>` no longer exists in any container.
        Rewrite into:
            ...;
            if (c) { ... } else { <suffix> }
        """
        changed = True
        while changed:
            changed = False
            known_blocks = self._collect_all_blocks(self._root_body)

            for index, statement in enumerate(list(block.statements)):
                if not isinstance(statement, IRIf):
                    continue
                if statement.false_statement is not None:
                    continue
                if not isinstance(statement.true_statement, IRBlock):
                    continue

                true_block = statement.true_statement
                if not true_block.statements:
                    continue
                tail = true_block.statements[-1]
                if not isinstance(tail, IRJump):
                    continue
                if tail.target in known_blocks:
                    continue

                suffix = block.statements[index + 1 :]
                if not suffix:
                    continue
                if not self._statements_end_unreachable(suffix):
                    continue

                statement.true_statement = IRBlock(
                    statements=list(true_block.statements[:-1]),
                    start_address=true_block.start_address,
                )
                statement.false_statement = IRBlock(
                    statements=list(suffix),
                    start_address=self._new_block_start_address(suffix[0]),
                )
                block.statements = block.statements[: index + 1]

                changed = True
                break

    def _count_jump_target_references_in_block(
        self,
        block: IRBlock,
        target: IRBlock,
    ) -> int:
        count = 0
        for statement in block.statements:
            count += self._count_jump_target_references_in_statement(
                statement=statement,
                target=target,
            )
        return count

    def _count_jump_target_references_in_statement(
        self,
        statement: IRStatement,
        target: IRBlock,
    ) -> int:
        count = 0

        if isinstance(statement, IRJump):
            return 1 if statement.target == target else 0

        if isinstance(statement, IRIf):
            count += self._count_jump_target_references_in_statement(
                statement=statement.true_statement,
                target=target,
            )
            if statement.false_statement is not None:
                count += self._count_jump_target_references_in_statement(
                    statement=statement.false_statement,
                    target=target,
                )
            return count

        if isinstance(statement, IRBlock):
            for nested in statement.statements:
                count += self._count_jump_target_references_in_statement(
                    statement=nested,
                    target=target,
                )
            return count

        if isinstance(statement, IRBlockContainer):
            for nested_block in statement.blocks:
                for nested in nested_block.statements:
                    count += self._count_jump_target_references_in_statement(
                        statement=nested,
                        target=target,
                    )
            return count

        if isinstance(statement, IRHighLevelSwitch):
            for section in statement.sections:
                count += self._count_jump_target_references_in_statement(
                    statement=section.body,
                    target=target,
                )
            return count

        if isinstance(statement, (IRHighLevelWhile, IRHighLevelDoWhile)):
            for nested_block in statement.body.blocks:
                for nested in nested_block.statements:
                    count += self._count_jump_target_references_in_statement(
                        statement=nested,
                        target=target,
                    )

        return count

    def _remove_first_jump_target_reference(
        self,
        statement: IRStatement,
        target: IRBlock,
    ) -> bool:
        if isinstance(statement, IRBlock):
            for index, nested in enumerate(list(statement.statements)):
                if isinstance(nested, IRJump) and nested.target == target:
                    statement.statements.pop(index)
                    return True
                if self._remove_first_jump_target_reference(
                    statement=nested,
                    target=target,
                ):
                    return True
            return False

        if isinstance(statement, IRIf):
            if self._remove_first_jump_target_reference(
                statement=statement.true_statement,
                target=target,
            ):
                return True
            if statement.false_statement is not None:
                if self._remove_first_jump_target_reference(
                    statement=statement.false_statement,
                    target=target,
                ):
                    return True
            return False

        if isinstance(statement, IRBlockContainer):
            for nested_block in statement.blocks:
                for nested in nested_block.statements:
                    if self._remove_first_jump_target_reference(
                        statement=nested,
                        target=target,
                    ):
                        return True
            return False

        if isinstance(statement, IRHighLevelSwitch):
            for section in statement.sections:
                if self._remove_first_jump_target_reference(
                    statement=section.body,
                    target=target,
                ):
                    return True
            return False

        if isinstance(statement, (IRHighLevelWhile, IRHighLevelDoWhile)):
            for nested_block in statement.body.blocks:
                for nested in nested_block.statements:
                    if self._remove_first_jump_target_reference(
                        statement=nested,
                        target=target,
                    ):
                        return True
            return False

        return False

    def _collect_all_blocks(self, container: IRBlockContainer) -> set[IRBlock]:
        blocks: set[IRBlock] = set()

        def visit_statement(statement: IRStatement) -> None:
            if isinstance(statement, IRIf):
                visit_statement(statement.true_statement)
                if statement.false_statement is not None:
                    visit_statement(statement.false_statement)
                return

            if isinstance(statement, IRBlock):
                for nested in statement.statements:
                    visit_statement(nested)
                return

            if isinstance(statement, IRBlockContainer):
                visit_container(statement)
                return

            if isinstance(statement, (IRHighLevelWhile, IRHighLevelDoWhile)):
                visit_container(statement.body)
                return

            if isinstance(statement, IRHighLevelSwitch):
                for section in statement.sections:
                    visit_statement(section.body)

        def visit_container(current: IRBlockContainer) -> None:
            for current_block in current.blocks:
                blocks.add(current_block)
                for nested_statement in current_block.statements:
                    visit_statement(nested_statement)

        visit_container(container)
        return blocks

    @staticmethod
    def _statement_start_address(statement: IRStatement) -> int:
        if isinstance(statement, IRBlock):
            return statement.start_address
        return -1

    def _new_block_start_address(self, first_statement: IRStatement) -> int:
        start = self._statement_start_address(first_statement)
        if start != -1:
            return start
        return self._next_synthetic_block_address()

    def _next_synthetic_block_address(self) -> int:
        key = "_synthetic_block_addr"
        current = self._context.metadata.get(key)
        if not isinstance(current, int):
            current = -1
        self._context.metadata[key] = current - 1
        return current - 1

    def _simplify_late_true_target_if_gotos(
        self,
        body: IRBlockContainer,
    ) -> None:
        """
        Simplify shape:
            A: if (c) goto T; goto F;
        where F is the immediate next block, and T appears later in container.

        Rewrite:
            A: if (c) { <T statements>; [jump to T fallthrough successor] }
            F: ...

        T must have one incoming edge (from A) to avoid duplicating shared blocks.
        """
        changed = True
        while changed:
            changed = False
            incoming = self._compute_incoming_counts(body)

            for block_index, block in enumerate(list(body.blocks)):
                if block_index >= len(body.blocks):
                    break
                if len(block.statements) < 2:
                    continue

                if_stmt = block.statements[-2]
                fallback_jump = block.statements[-1]
                if not isinstance(if_stmt, IRIf) or if_stmt.false_statement is not None:
                    continue
                if not isinstance(fallback_jump, IRJump):
                    continue

                if block_index + 1 >= len(body.blocks):
                    continue
                false_target = fallback_jump.target
                if body.blocks[block_index + 1] is not false_target:
                    continue

                true_target = self._as_jump_target(if_stmt.true_statement)
                if true_target is None:
                    continue
                if true_target not in body.blocks:
                    continue
                if true_target is false_target:
                    continue
                if incoming.get(true_target, 0) != 1:
                    continue

                true_index = body.blocks.index(true_target)
                if true_index <= block_index + 1:
                    continue

                inlined_statements = list(true_target.statements)
                if not self._statements_end_unreachable(inlined_statements):
                    if true_index + 1 >= len(body.blocks):
                        continue
                    successor = body.blocks[true_index + 1]
                    inlined_statements.append(IRJump(target=successor))

                if_stmt.true_statement = IRBlock(
                    statements=inlined_statements,
                    start_address=true_target.start_address,
                )
                block.statements.pop()  # remove goto F (now falls through to F)

                if true_target in body.blocks:
                    body.blocks.remove(true_target)

                changed = True
                break

    def _simplify_nested_if_goto_fallbacks(
        self,
        body: IRBlockContainer,
    ) -> None:
        """
        Simplify nested branch blocks of form:
            { ...; if (c) <branch>; goto F; }
        by inlining fallback target block F into `else` branch when F is private.
        """
        changed = True
        while changed:
            changed = False
            incoming = self._compute_incoming_counts(body)
            nested_blocks = self._collect_nested_ir_blocks(body)

            for nested in nested_blocks:
                if len(nested.statements) < 2:
                    continue

                if_stmt = nested.statements[-2]
                fallback_jump = nested.statements[-1]
                if not isinstance(if_stmt, IRIf) or if_stmt.false_statement is not None:
                    continue
                if not isinstance(fallback_jump, IRJump):
                    continue

                fallback_target = fallback_jump.target
                if fallback_target not in body.blocks:
                    continue
                if incoming.get(fallback_target, 0) != 1:
                    continue

                fallback_index = body.blocks.index(fallback_target)
                inlined = list(fallback_target.statements)
                if not self._statements_end_unreachable(inlined):
                    if fallback_index + 1 >= len(body.blocks):
                        continue
                    successor = body.blocks[fallback_index + 1]
                    inlined.append(IRJump(target=successor))

                if_stmt.false_statement = IRBlock(
                    statements=inlined,
                    start_address=fallback_target.start_address,
                )
                nested.statements.pop()  # remove trailing goto fallback

                if fallback_target in body.blocks:
                    body.blocks.remove(fallback_target)

                changed = True
                break

    def _collect_nested_ir_blocks(
        self,
        body: IRBlockContainer,
    ) -> list[IRBlock]:
        nested: list[IRBlock] = []

        def visit_statement(statement: IRStatement) -> None:
            if isinstance(statement, IRIf):
                visit_statement(statement.true_statement)
                if statement.false_statement is not None:
                    visit_statement(statement.false_statement)
                return

            if isinstance(statement, IRBlock):
                nested.append(statement)
                for child in statement.statements:
                    visit_statement(child)
                return

            if isinstance(statement, (IRHighLevelWhile, IRHighLevelDoWhile)):
                for child_block in statement.body.blocks:
                    for child in child_block.statements:
                        visit_statement(child)
                return

            if isinstance(statement, IRHighLevelSwitch):
                for section in statement.sections:
                    visit_statement(section.body)
                return

            if isinstance(statement, IRBlockContainer):
                for child_block in statement.blocks:
                    for child in child_block.statements:
                        visit_statement(child)

        for block in body.blocks:
            for statement in block.statements:
                visit_statement(statement)

        return nested

    @staticmethod
    def _region_has_no_extra_explicit_entries(
        region_blocks: list[IRBlock],
        incoming: dict[IRBlock, int],
    ) -> bool:
        for region_block in region_blocks[1:]:
            if incoming.get(region_block, 0) != 0:
                return False
        return True

    def _region_is_mergeable(
        self,
        region_blocks: list[IRBlock],
        region_end: IRBlock,
    ) -> bool:
        for index, region_block in enumerate(region_blocks):
            if not region_block.statements:
                continue
            tail = region_block.statements[-1]
            is_last = index == len(region_blocks) - 1
            if is_last and isinstance(tail, IRJump) and tail.target is region_end:
                continue
            if self._statement_is_unreachable_endpoint(tail):
                return False
        return True

    def _compute_incoming_counts(
        self,
        body: IRBlockContainer,
    ) -> dict[IRBlock, int]:
        counts: dict[IRBlock, int] = {block: 0 for block in body.blocks}
        for block in body.blocks:
            for statement in block.statements:
                self._count_targets_in_statement(statement, counts)
        return counts

    def _count_targets_in_statement(
        self,
        statement: IRStatement,
        counts: dict[IRBlock, int],
    ) -> None:
        if isinstance(statement, IRJump):
            if statement.target in counts:
                counts[statement.target] = counts.get(statement.target, 0) + 1
            return

        if isinstance(statement, IRIf):
            self._count_targets_in_statement(statement.true_statement, counts)
            if statement.false_statement is not None:
                self._count_targets_in_statement(statement.false_statement, counts)
            return

        if isinstance(statement, IRBlock):
            for nested in statement.statements:
                self._count_targets_in_statement(nested, counts)
            return

        if isinstance(statement, IRBlockContainer):
            for nested_block in statement.blocks:
                for nested in nested_block.statements:
                    self._count_targets_in_statement(nested, counts)
            return

    @staticmethod
    def _statements_end_unreachable(statements: list[IRStatement]) -> bool:
        if not statements:
            return False
        return StructuredControlFlowCleanupTransform._statement_is_unreachable_endpoint(
            statements[-1]
        )

    @staticmethod
    def _statement_is_unreachable_endpoint(statement: IRStatement) -> bool:
        if isinstance(statement, (IRJump, IRLeave, IRReturn, IRSwitch)):
            return True

        if isinstance(statement, IRIf):
            if statement.false_statement is None:
                return False
            return (
                StructuredControlFlowCleanupTransform._statement_is_unreachable_endpoint(
                    statement.true_statement
                )
                and (
                    StructuredControlFlowCleanupTransform._statement_is_unreachable_endpoint(
                        statement.false_statement
                    )
                )
            )

        if isinstance(statement, IRBlock):
            if not statement.statements:
                return False
            return StructuredControlFlowCleanupTransform._statement_is_unreachable_endpoint(
                statement.statements[-1]
            )

        return False

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
