from __future__ import annotations

from typing import Dict, Optional, Set

from udon_decompiler.analysis.ir.control_flow_node import ControlFlowNode
from udon_decompiler.analysis.ir.dominance import (
    compute_dominance,
    mark_nodes_with_reachable_exits,
)
from udon_decompiler.analysis.ir.nodes import (
    IRBlock,
    IRBlockContainer,
    IRHighLevelDoWhile,
    IRHighLevelSwitch,
    IRHighLevelWhile,
    IRIf,
    IRJump,
    IRLeave,
    IRReturn,
    IRStatement,
    IRSwitch,
)


class ControlFlowGraph:
    """
    ILSpy-style control-flow graph for one IRBlockContainer.

    A CFG snapshot is built once before block transforms for the container.
    Later transforms may move/reorder blocks, but this graph keeps the original
    node identity/mapping for algorithms that rely on a stable snapshot.
    """

    def __init__(
        self,
        container: IRBlockContainer,
        function_body: IRBlockContainer,
    ) -> None:
        self.container = container
        self._function_body = function_body

        self.cfg: list[ControlFlowNode] = []
        self._dict: Dict[IRBlock, ControlFlowNode] = {}

        self._node_has_direct_exit_out_of_container: list[bool] = []
        self._node_has_reachable_exit: list[bool] = []

        self._container_block_to_index: Dict[IRBlock, int] = {
            block: index for index, block in enumerate(container.blocks)
        }
        self._nested_descendant_blocks = self._collect_nested_descendant_blocks(
            container
        )

        self._initialize_nodes()
        self._create_edges()

        if self.cfg:
            entry = self.cfg[0]
            compute_dominance(entry)
            self._node_has_reachable_exit = mark_nodes_with_reachable_exits(self.cfg)
            leaving = self._find_nodes_with_exits_out_of_container()
            self._node_has_reachable_exit = [
                self._node_has_reachable_exit[i] or leaving[i]
                for i in range(len(self.cfg))
            ]

    def get_node(self, block: IRBlock) -> ControlFlowNode:
        return self._dict[block]

    def has_reachable_exit(self, node: ControlFlowNode) -> bool:
        self._validate_node(node)
        return self._node_has_reachable_exit[node.user_index]

    def has_direct_exit_out_of_container(self, node: ControlFlowNode) -> bool:
        self._validate_node(node)
        return self._node_has_direct_exit_out_of_container[node.user_index]

    def _validate_node(self, node: ControlFlowNode) -> None:
        if node.user_index < 0 or node.user_index >= len(self.cfg):
            raise IndexError("ControlFlowNode index out of range")
        if self.cfg[node.user_index] is not node:
            raise ValueError("ControlFlowNode does not belong to this CFG")

    def _initialize_nodes(self) -> None:
        for index, block in enumerate(self.container.blocks):
            node = ControlFlowNode(user_index=index, block=block)
            self.cfg.append(node)
            self._dict[block] = node

        self._node_has_direct_exit_out_of_container = [False] * len(self.cfg)
        self._node_has_reachable_exit = [False] * len(self.cfg)

    def _create_edges(self) -> None:
        for index, block in enumerate(self.container.blocks):
            source_node = self.cfg[index]
            descendant_container_ids = self._collect_descendant_container_ids(block)
            for statement in block.statements:
                self._process_statement(
                    source_node=source_node,
                    source_block=block,
                    statement=statement,
                    descendant_container_ids=descendant_container_ids,
                )
            self._add_fallthrough_edge_if_needed(
                source_node=source_node,
                block=block,
                block_index=index,
            )

    def _add_fallthrough_edge_if_needed(
        self,
        source_node: ControlFlowNode,
        block: IRBlock,
        block_index: int,
    ) -> None:
        if self._block_has_unreachable_endpoint(block):
            return

        next_index = block_index + 1
        if next_index < len(self.cfg):
            source_node.add_edge_to(self.cfg[next_index])
            return

        # Falling off the end of the current container exits the region.
        self._node_has_direct_exit_out_of_container[source_node.user_index] = True

    def _process_statement(
        self,
        source_node: ControlFlowNode,
        source_block: IRBlock,
        statement: IRStatement,
        descendant_container_ids: Set[int],
    ) -> None:
        if isinstance(statement, IRJump):
            self._process_branch_target(source_node, statement.target)
            return

        if isinstance(statement, IRIf):
            self._process_statement(
                source_node,
                source_block,
                statement.true_statement,
                descendant_container_ids,
            )
            if statement.false_statement is not None:
                self._process_statement(
                    source_node,
                    source_block,
                    statement.false_statement,
                    descendant_container_ids,
                )
            return

        if isinstance(statement, IRSwitch):
            for target in statement.cases.values():
                self._process_branch_target(source_node, target)
            if statement.default_target is not None:
                self._process_branch_target(source_node, statement.default_target)
            return

        if isinstance(statement, IRLeave):
            if self._is_leave_out_of_container(
                source_block=source_block,
                leave=statement,
                descendant_container_ids=descendant_container_ids,
            ):
                self._node_has_direct_exit_out_of_container[source_node.user_index] = (
                    True
                )
            return

        if isinstance(statement, IRBlockContainer):
            for nested_block in statement.blocks:
                for nested_statement in nested_block.statements:
                    self._process_statement(
                        source_node,
                        source_block,
                        nested_statement,
                        descendant_container_ids,
                    )

    def _block_has_unreachable_endpoint(self, block: IRBlock) -> bool:
        if not block.statements:
            return False
        return self._statement_has_unreachable_endpoint(block.statements[-1])

    def _statement_has_unreachable_endpoint(self, statement: IRStatement) -> bool:
        if isinstance(statement, (IRJump, IRLeave, IRReturn, IRSwitch)):
            return True

        if isinstance(statement, IRIf):
            if statement.false_statement is None:
                return False
            return (
                self._statement_has_unreachable_endpoint(statement.true_statement)
                and self._statement_has_unreachable_endpoint(
                    statement.false_statement
                )
            )

        if isinstance(statement, IRBlock):
            if not statement.statements:
                return False
            return self._statement_has_unreachable_endpoint(statement.statements[-1])

        if isinstance(statement, IRHighLevelSwitch):
            return True

        if isinstance(statement, (IRHighLevelWhile, IRHighLevelDoWhile)):
            return False

        return False

    def _process_branch_target(
        self,
        source_node: ControlFlowNode,
        target: IRBlock,
    ) -> None:
        if target in self._container_block_to_index:
            target_index = self._container_block_to_index[target]
            source_node.add_edge_to(self.cfg[target_index])
            return

        # Internal control flow within nested containers is ignored for this CFG.
        if target in self._nested_descendant_blocks:
            return

        self._node_has_direct_exit_out_of_container[source_node.user_index] = True

    def _is_leave_out_of_container(
        self,
        source_block: IRBlock,
        leave: IRLeave,
        descendant_container_ids: Set[int],
    ) -> bool:
        # Returning from function is not considered a reachable exit.
        if leave.target_container is self._function_body:
            return False

        # Leave into containers nested inside this block is internal control flow.
        if id(leave.target_container) in descendant_container_ids:
            return False

        # Everything else exits the current container region.
        _ = source_block
        return True

    def _find_nodes_with_exits_out_of_container(self) -> list[bool]:
        leaving = [False] * len(self.cfg)
        for node in self.cfg:
            if leaving[node.user_index]:
                continue
            if not self._node_has_direct_exit_out_of_container[node.user_index]:
                continue

            current: Optional[ControlFlowNode] = node
            while current is not None:
                if leaving[current.user_index]:
                    break
                leaving[current.user_index] = True
                current = current.immediate_dominator

        return leaving

    def _collect_nested_descendant_blocks(
        self,
        container: IRBlockContainer,
    ) -> Set[IRBlock]:
        descendants: set[IRBlock] = set()
        visited_containers: set[int] = set()

        def visit_statement(statement: IRStatement) -> None:
            if not isinstance(statement, IRBlockContainer):
                return
            visit_container(statement)

        def visit_container(nested: IRBlockContainer) -> None:
            nested_id = id(nested)
            if nested_id in visited_containers:
                return
            visited_containers.add(nested_id)

            for nested_block in nested.blocks:
                descendants.add(nested_block)
                for nested_statement in nested_block.statements:
                    visit_statement(nested_statement)

        for block in container.blocks:
            for statement in block.statements:
                visit_statement(statement)

        return descendants

    def _collect_descendant_container_ids(self, block: IRBlock) -> Set[int]:
        result: set[int] = set()
        visited_containers: set[int] = set()

        def visit_statement(statement: IRStatement) -> None:
            if not isinstance(statement, IRBlockContainer):
                return

            nested = statement
            nested_id = id(nested)
            if nested_id in visited_containers:
                return
            visited_containers.add(nested_id)

            result.add(nested_id)
            for nested_block in nested.blocks:
                for nested_statement in nested_block.statements:
                    visit_statement(nested_statement)

        for statement in block.statements:
            visit_statement(statement)

        return result
