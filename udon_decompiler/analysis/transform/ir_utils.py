from __future__ import annotations

from collections import deque
from typing import Dict, Iterator, List, Optional, Set

from udon_decompiler.analysis.ir.nodes import (
    IRBlock,
    IRBlockContainer,
    IRFunction,
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


def _build_successor_map(container: IRBlockContainer) -> Dict[IRBlock, List[IRBlock]]:
    block_set = set(container.blocks)
    successors: Dict[IRBlock, List[IRBlock]] = {block: [] for block in container.blocks}

    for block in container.blocks:
        terminator = get_block_terminator(block)
        if terminator is None:
            continue
        for target in iter_block_targets(terminator):
            if target in block_set:
                successors[block].append(target)

    return successors


def reachable_blocks(container: IRBlockContainer) -> List[IRBlock]:
    if container.entry_block is None:
        return []

    successors = _build_successor_map(container)
    entry = container.entry_block

    visited: Set[IRBlock] = set()
    queue: deque[IRBlock] = deque([entry])
    while queue:
        block = queue.popleft()
        if block in visited:
            continue
        visited.add(block)
        for succ in successors.get(block, []):
            if succ not in visited:
                queue.append(succ)
    return [b for b in container.blocks if b in visited]


def iter_block_containers(function: IRFunction) -> Iterator[IRBlockContainer]:
    visited: set[int] = set()
    queue: deque[IRBlockContainer] = deque([function.body])

    while queue:
        container = queue.popleft()
        container_id = id(container)
        if container_id in visited:
            continue
        visited.add(container_id)
        yield container

        for block in container.blocks:
            for statement in block.statements:
                queue.extend(_iter_nested_containers(statement))


def _iter_nested_containers(statement: IRStatement) -> Iterator[IRBlockContainer]:
    if isinstance(statement, IRBlockContainer):
        yield statement
        for block in statement.blocks:
            for nested_statement in block.statements:
                yield from _iter_nested_containers(nested_statement)
        return

    if isinstance(statement, IRBlock):
        for nested_statement in statement.statements:
            yield from _iter_nested_containers(nested_statement)
        return

    if isinstance(statement, IRIf):
        yield from _iter_nested_containers(statement.true_statement)
        if statement.false_statement is not None:
            yield from _iter_nested_containers(statement.false_statement)
        return

    if isinstance(statement, IRHighLevelSwitch):
        for section in statement.sections:
            yield from _iter_nested_containers(section.body)
        return

    if isinstance(statement, (IRHighLevelWhile, IRHighLevelDoWhile)):
        yield from _iter_nested_containers(statement.body)
