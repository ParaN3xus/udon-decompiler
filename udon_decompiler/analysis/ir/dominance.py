from __future__ import annotations

from typing import List, Optional, Sequence

from udon_decompiler.analysis.ir.control_flow_node import ControlFlowNode


def compute_dominance(entry_point: ControlFlowNode) -> None:
    """
    Compute immediate dominators and dominator tree from entry point.

    Port of ILSpy's Dominance.ComputeDominance for ControlFlowNode.
    """

    nodes: list[ControlFlowNode] = []
    entry_point.traverse_post_order(lambda n: n.successors, nodes.append)
    if not nodes:
        return

    for index, node in enumerate(nodes):
        node.post_order_number = index

    entry_point.immediate_dominator = entry_point
    changed = True
    while changed:
        changed = False

        # Reverse post-order, excluding the entry point itself.
        for index in range(len(nodes) - 2, -1, -1):
            node = nodes[index]
            new_idom: Optional[ControlFlowNode] = None
            for pred in node.predecessors:
                if pred.immediate_dominator is None:
                    continue
                if new_idom is None:
                    new_idom = pred
                else:
                    new_idom = find_common_dominator(pred, new_idom)

            if new_idom is None:
                continue

            if node.immediate_dominator is not new_idom:
                node.immediate_dominator = new_idom
                changed = True

    for node in nodes:
        if node.immediate_dominator is not None:
            node.dominator_tree_children = []

    entry_point.immediate_dominator = None
    for node in nodes:
        if node.immediate_dominator is not None:
            assert node.immediate_dominator.dominator_tree_children is not None
            node.immediate_dominator.dominator_tree_children.append(node)
        node.visited = False


def find_common_dominator(a: ControlFlowNode, b: ControlFlowNode) -> ControlFlowNode:
    """
    Return the common ancestor of a and b in the dominator tree.

    Precondition: both nodes are in the same dominator tree.
    """

    while a is not b:
        while a.post_order_number < b.post_order_number:
            assert a.immediate_dominator is not None
            a = a.immediate_dominator
        while b.post_order_number < a.post_order_number:
            assert b.immediate_dominator is not None
            b = b.immediate_dominator
    return a


def mark_nodes_with_reachable_exits(
    cfg_nodes: Sequence[ControlFlowNode],
) -> List[bool]:
    """
    Equivalent to ILSpy Dominance.MarkNodesWithReachableExits.

    result[i] == true iff cfg_nodes[i] has a reachable node that is not dominated
    by cfg_nodes[i].
    """

    result = [False] * len(cfg_nodes)

    for join_node in cfg_nodes:
        if not join_node.is_reachable:
            continue

        has_multiple_inputs = len(join_node.predecessors) >= 2
        has_root_extra_input = (
            len(join_node.predecessors) >= 1
            and join_node.immediate_dominator is None
        )
        if not has_multiple_inputs and not has_root_extra_input:
            continue

        for pred in join_node.predecessors:
            runner: Optional[ControlFlowNode] = pred
            while (
                runner is not None
                and runner is not join_node.immediate_dominator
                and runner is not join_node
            ):
                result[runner.user_index] = True
                runner = runner.immediate_dominator

    return result
