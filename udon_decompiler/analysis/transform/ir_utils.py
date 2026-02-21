from __future__ import annotations

from collections import defaultdict, deque
from typing import DefaultDict, Dict, Iterator, List, Optional, Set

import networkx as nx

from udon_decompiler.analysis.ir.nodes import (
    IRBlock,
    IRBlockContainer,
    IRFunction,
    IRIf,
    IRJump,
    IRLeave,
    IRReturn,
    IRStatement,
    IRSwitch,
)


def is_terminator(statement: IRStatement) -> bool:
    return isinstance(statement, (IRIf, IRJump, IRLeave, IRReturn, IRSwitch))


def get_block_terminator(block: IRBlock) -> Optional[IRStatement]:
    if not block.statements:
        return None
    last = block.statements[-1]
    if is_terminator(last):
        return last
    return None


def iter_block_targets(statement: IRStatement) -> Iterator[IRBlock]:
    if isinstance(statement, IRJump):
        yield statement.target
        return

    if isinstance(statement, IRIf):
        yield from iter_block_targets(statement.true_statement)
        if statement.false_statement is not None:
            yield from iter_block_targets(statement.false_statement)
        return

    if isinstance(statement, IRSwitch):
        for block in statement.cases.values():
            yield block
        if statement.default_target is not None:
            yield statement.default_target


def build_container_cfg(container: IRBlockContainer) -> nx.DiGraph:
    graph = nx.DiGraph()
    for block in container.blocks:
        graph.add_node(block)

    for block in container.blocks:
        terminator = get_block_terminator(block)
        if terminator is None:
            continue
        for target in iter_block_targets(terminator):
            if target in graph:
                graph.add_edge(block, target)

    return graph


def reachable_blocks(container: IRBlockContainer) -> List[IRBlock]:
    if container.entry_block is None:
        return []
    graph = build_container_cfg(container)
    entry = container.entry_block
    if entry not in graph:
        return []

    visited: Set[IRBlock] = set()
    queue: deque[IRBlock] = deque([entry])
    while queue:
        block = queue.popleft()
        if block in visited:
            continue
        visited.add(block)
        for succ in graph.successors(block):
            if succ not in visited:
                queue.append(succ)
    return [b for b in container.blocks if b in visited]


def build_dominator_tree_children(
    immediate_dominators: Dict[IRBlock, IRBlock], entry: IRBlock
) -> Dict[IRBlock, List[IRBlock]]:
    children: DefaultDict[IRBlock, List[IRBlock]] = defaultdict(list)
    for block, idom in immediate_dominators.items():
        if block == entry:
            continue
        children[idom].append(block)
    for blocks in children.values():
        blocks.sort(key=lambda b: b.start_address)
    return dict(children)


def iter_block_containers(function: IRFunction) -> Iterator[IRBlockContainer]:
    # Current IR has a single top-level body container.
    # Keep this helper to match ILSpy-style multi-container traversal.
    yield function.body
