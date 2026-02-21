from __future__ import annotations

from typing import List, Optional, Set, cast

from udon_decompiler.analysis.ir.control_flow_node import ControlFlowNode
from udon_decompiler.analysis.ir.dominance import (
    compute_dominance,
    find_common_dominator,
)
from udon_decompiler.analysis.ir.nodes import (
    IRBlock,
    IRBlockContainer,
    IRIf,
    IRJump,
    IRLeave,
    IRStatement,
    IRSwitch,
)
from udon_decompiler.analysis.transform.pass_base import (
    BlockTransformContext,
    IBlockTransform,
)


class LoopDetection(IBlockTransform):
    """Port of ILSpy LoopDetection for current Python IR."""

    _NO_EXIT_POINT = object()

    def __init__(self) -> None:
        self.context: Optional[BlockTransformContext] = None
        self.current_container: Optional[IRBlockContainer] = None

    def run(self, block: IRBlock, context: BlockTransformContext) -> None:
        if block not in context.container.blocks:
            return

        self.context = context
        self.current_container = context.control_flow_graph.container

        head = context.control_flow_node
        if head is None:
            return
        if head.user_data is not block:
            head = context.control_flow_graph.get_node(block)

        loop: Optional[list[ControlFlowNode]] = None
        for pred in head.predecessors:
            if not head.dominates(pred):
                continue

            if loop is None:
                loop = [head]
                head.visited = True

            pred.traverse_pre_order(lambda n: n.predecessors, loop.append)

        if loop is None:
            return

        self._include_nested_containers(loop)

        exit_point = self._extend_loop(head, loop)

        loop.sort(key=lambda n: -n.post_order_number)
        for node in loop:
            node.visited = False
            assert head.dominates(node) or not node.is_reachable

        self._construct_loop(loop, exit_point)
        context.mark_dirty()

    def _include_nested_containers(self, loop: list[ControlFlowNode]) -> None:
        index = 0
        while index < len(loop):
            self._include_block(loop[index], loop)
            index += 1

    def _include_block(
        self,
        node: ControlFlowNode,
        loop: list[ControlFlowNode],
    ) -> None:
        assert self.context is not None

        block = cast(IRBlock, node.user_data)
        if not block.statements or not isinstance(
            block.statements[0], IRBlockContainer
        ):
            return

        nested_container = block.statements[0]
        if nested_container.entry_block is not None:
            try:
                entry_node = self.context.control_flow_graph.get_node(
                    nested_container.entry_block
                )
            except KeyError:
                entry_node = None
            if entry_node is not None:
                self._include_block(entry_node, loop)

        for nested_block in nested_container.blocks[1:]:
            try:
                nested_node = self.context.control_flow_graph.get_node(nested_block)
            except KeyError:
                continue
            if nested_node.visited:
                continue
            nested_node.visited = True
            loop.append(nested_node)

    def _extend_loop(
        self,
        loop_head: ControlFlowNode,
        loop: list[ControlFlowNode],
    ) -> Optional[ControlFlowNode]:
        exit_point = self._find_exit_point(loop_head, loop)
        if exit_point is not None:
            self._add_dominated_until_exit(loop_head, loop, exit_point)
            if exit_point is self._NO_EXIT_POINT:
                return None
            return cast(ControlFlowNode, exit_point)

        self._extend_loop_heuristic(loop_head, loop, loop_head)
        return None

    def _find_exit_point(
        self,
        loop_head: ControlFlowNode,
        natural_loop: list[ControlFlowNode],
    ) -> Optional[object]:
        assert self.context is not None
        cfg = self.context.control_flow_graph

        if not cfg.has_reachable_exit(loop_head):
            best_exit = self._pick_exit_point(loop_head)
            if best_exit is not None:
                return best_exit
            return self._NO_EXIT_POINT

        rev_cfg, exit_node_arity = self._prepare_reverse_cfg(
            loop_head,
            treat_back_edges_as_exits=False,
        )

        common_ancestor = rev_cfg[loop_head.user_index]
        if not common_ancestor.is_reachable:
            return None

        for node in natural_loop:
            rev_node = rev_cfg[node.user_index]
            if rev_node.is_reachable:
                common_ancestor = find_common_dominator(common_ancestor, rev_node)

        original_cfg = cfg.cfg
        while common_ancestor.user_index >= 0:
            exit_point = original_cfg[common_ancestor.user_index]
            if (
                not exit_point.visited
                and self._validate_exit_point(loop_head, exit_point)
            ):
                return exit_point
            if common_ancestor.immediate_dominator is None:
                break
            common_ancestor = common_ancestor.immediate_dominator

        if exit_node_arity > 1:
            return None
        return self._NO_EXIT_POINT

    def _prepare_reverse_cfg(
        self,
        loop_head: ControlFlowNode,
        treat_back_edges_as_exits: bool,
    ) -> tuple[list[ControlFlowNode], int]:
        assert self.context is not None
        cfg = self.context.control_flow_graph
        nodes = cfg.cfg

        rev: list[ControlFlowNode] = [
            ControlFlowNode(user_index=i, user_data=nodes[i].user_data)
            for i in range(len(nodes))
        ]

        node_treated_as_exit: Optional[ControlFlowNode] = None
        multiple_exit_nodes = False

        exit_node = ControlFlowNode(user_index=-1, user_data=None)
        rev.append(exit_node)

        for i, node in enumerate(nodes):
            if not loop_head.dominates(node):
                continue

            for succ in node.successors:
                if loop_head.dominates(succ) and (
                    not treat_back_edges_as_exits or loop_head is not succ
                ):
                    rev[succ.user_index].add_edge_to(rev[i])
                else:
                    if node_treated_as_exit is None:
                        node_treated_as_exit = succ
                    if node_treated_as_exit is not succ:
                        multiple_exit_nodes = True
                    exit_node.add_edge_to(rev[i])

            if cfg.has_direct_exit_out_of_container(node):
                exit_node.add_edge_to(rev[i])

        if multiple_exit_nodes:
            exit_node_arity = 2
        elif node_treated_as_exit is not None:
            exit_node_arity = 1
        else:
            exit_node_arity = 0

        compute_dominance(exit_node)
        return rev, exit_node_arity

    def _validate_exit_point(
        self,
        loop_head: ControlFlowNode,
        exit_point: ControlFlowNode,
    ) -> bool:
        assert self.context is not None
        cfg = self.context.control_flow_graph

        def is_valid(node: ControlFlowNode) -> bool:
            if not cfg.has_reachable_exit(node):
                return True

            for succ in node.successors:
                if (
                    succ is not loop_head
                    and loop_head.dominates(succ)
                    and not exit_point.dominates(succ)
                ):
                    return False

            for child in node.dominator_tree_children or []:
                if not is_valid(child):
                    return False
            return True

        return is_valid(exit_point)

    def _pick_exit_point(self, loop_head: ControlFlowNode) -> Optional[ControlFlowNode]:
        assert self.context is not None
        cfg = self.context.control_flow_graph

        best_node: Optional[ControlFlowNode] = None
        best_start_address = -1

        def walk(node: ControlFlowNode) -> None:
            nonlocal best_node, best_start_address

            block = cast(IRBlock, node.user_data)
            if (
                block.start_address > best_start_address
                and not cfg.has_reachable_exit(node)
                and self.current_container is not None
                and block in self.current_container.blocks
            ):
                best_node = node
                best_start_address = block.start_address
                return

            for child in node.dominator_tree_children or []:
                walk(child)

        for child in loop_head.dominator_tree_children or []:
            walk(child)

        return best_node

    def _add_dominated_until_exit(
        self,
        loop_head: ControlFlowNode,
        loop: list[ControlFlowNode],
        exit_point: object,
    ) -> None:
        stack: list[ControlFlowNode] = [loop_head]

        while stack:
            node = stack.pop()
            if node is not exit_point and not node.visited:
                node.visited = True
                loop.append(node)

            if node is exit_point:
                continue

            children = node.dominator_tree_children or []
            for child in reversed(children):
                stack.append(child)

    def _extend_loop_heuristic(
        self,
        loop_head: ControlFlowNode,
        loop: list[ControlFlowNode],
        candidate: ControlFlowNode,
    ) -> None:
        _ = loop_head

        if not candidate.visited:
            additional_nodes: list[ControlFlowNode] = []
            candidate.traverse_pre_order(
                lambda n: n.predecessors,
                additional_nodes.append,
            )

            new_exit_points = {
                succ
                for node in additional_nodes
                for succ in node.successors
                if not succ.visited
            }

            for node in additional_nodes:
                node.visited = False

            removed_exit_points = sum(
                1 for node in additional_nodes if self._is_exit_point(node)
            )
            added_exit_points = sum(
                1 for node in new_exit_points if not self._is_exit_point(node)
            )

            if removed_exit_points > added_exit_points:
                candidate.traverse_pre_order(lambda n: n.predecessors, loop.append)

        for child in candidate.dominator_tree_children or []:
            self._extend_loop_heuristic(loop_head, loop, child)

    def _is_exit_point(self, node: ControlFlowNode) -> bool:
        if node.visited:
            return False
        return any(pred.visited for pred in node.predecessors)

    def _construct_loop(
        self,
        loop: list[ControlFlowNode],
        exit_point: Optional[ControlFlowNode],
    ) -> None:
        if not loop:
            return

        assert self.current_container is not None

        old_entry = cast(IRBlock, loop[0].user_data)
        exit_target = cast(
            Optional[IRBlock],
            exit_point.user_data if exit_point else None,
        )

        loop_container = IRBlockContainer(blocks=[])
        new_entry = IRBlock(
            statements=list(old_entry.statements),
            start_address=self._next_synthetic_block_address(),
        )
        loop_container.blocks.append(new_entry)

        old_entry.statements = [loop_container]
        if exit_target is not None:
            old_entry.statements.append(IRJump(target=exit_target))

        for node in loop[1:]:
            block = cast(IRBlock, node.user_data)
            if block in self.current_container.blocks:
                self.current_container.blocks.remove(block)
                loop_container.blocks.append(block)

        self._rewrite_container_control_flow(
            loop_container,
            old_entry,
            new_entry,
            exit_target,
        )

    def _rewrite_container_control_flow(
        self,
        loop_container: IRBlockContainer,
        old_entry: IRBlock,
        new_entry: IRBlock,
        exit_target: Optional[IRBlock],
    ) -> None:
        for block in loop_container.blocks:
            block.statements = [
                self._rewrite_statement(
                    statement=statement,
                    loop_container=loop_container,
                    old_entry=old_entry,
                    new_entry=new_entry,
                    exit_target=exit_target,
                )
                for statement in block.statements
            ]

    def _rewrite_statement(
        self,
        statement: IRStatement,
        loop_container: IRBlockContainer,
        old_entry: IRBlock,
        new_entry: IRBlock,
        exit_target: Optional[IRBlock],
    ) -> IRStatement:
        if isinstance(statement, IRJump):
            if statement.target is old_entry:
                return IRJump(target=new_entry)
            if exit_target is not None and statement.target is exit_target:
                return IRLeave(target_container=loop_container)
            return statement

        if isinstance(statement, IRIf):
            statement.true_statement = self._rewrite_statement(
                statement.true_statement,
                loop_container,
                old_entry,
                new_entry,
                exit_target,
            )
            if statement.false_statement is not None:
                statement.false_statement = self._rewrite_statement(
                    statement.false_statement,
                    loop_container,
                    old_entry,
                    new_entry,
                    exit_target,
                )
            return statement

        if isinstance(statement, IRSwitch):
            statement.cases = {
                value: (new_entry if target is old_entry else target)
                for value, target in statement.cases.items()
            }
            if statement.default_target is old_entry:
                statement.default_target = new_entry
            return statement

        if isinstance(statement, IRBlockContainer):
            self._rewrite_container_control_flow(
                statement,
                old_entry,
                new_entry,
                exit_target,
            )
            return statement

        return statement

    def _next_synthetic_block_address(self) -> int:
        assert self.context is not None

        key = "_synthetic_block_addr"
        current = self.context.metadata.get(key)
        if not isinstance(current, int):
            current = -1
        self.context.metadata[key] = current - 1
        return current
