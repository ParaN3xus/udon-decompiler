from __future__ import annotations

from dataclasses import dataclass, field
from typing import Callable, Iterable, Optional


@dataclass(eq=False)
class ControlFlowNode:
    """ILSpy-style control-flow node."""

    user_index: int
    user_data: object

    visited: bool = False
    post_order_number: int = -1

    immediate_dominator: Optional["ControlFlowNode"] = None
    dominator_tree_children: Optional[list["ControlFlowNode"]] = None

    predecessors: list["ControlFlowNode"] = field(default_factory=list)
    successors: list["ControlFlowNode"] = field(default_factory=list)

    @property
    def is_reachable(self) -> bool:
        return self.dominator_tree_children is not None

    def add_edge_to(self, target: "ControlFlowNode") -> None:
        self.successors.append(target)
        target.predecessors.append(self)

    def traverse_pre_order(
        self,
        children: Callable[["ControlFlowNode"], Iterable["ControlFlowNode"]],
        visit_action: Callable[["ControlFlowNode"], None],
    ) -> None:
        if self.visited:
            return
        self.visited = True
        visit_action(self)
        for child in children(self):
            child.traverse_pre_order(children, visit_action)

    def traverse_post_order(
        self,
        children: Callable[["ControlFlowNode"], Iterable["ControlFlowNode"]],
        visit_action: Callable[["ControlFlowNode"], None],
    ) -> None:
        if self.visited:
            return
        self.visited = True
        for child in children(self):
            child.traverse_post_order(children, visit_action)
        visit_action(self)

    def dominates(self, node: "ControlFlowNode") -> bool:
        current: Optional[ControlFlowNode] = node
        while current is not None:
            if current is self:
                return True
            current = current.immediate_dominator
        return False
