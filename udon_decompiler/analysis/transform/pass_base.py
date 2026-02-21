from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any, Callable, Dict, Iterable, List, Optional, Sequence

import networkx as nx

from udon_decompiler.analysis.ir.nodes import (
    IRBlock,
    IRBlockContainer,
    IRClass,
    IRFunction,
)
from udon_decompiler.analysis.transform.ir_utils import (
    build_container_cfg,
    build_dominator_tree_children,
    iter_block_containers,
)
from udon_decompiler.models.program import UdonProgramData
from udon_decompiler.utils.logger import logger


class TransformStepper:
    """No-op stepper compatible with ILSpy-style grouped stepping."""

    def start_group(self, _description: str) -> None:
        return

    def end_group(self, keep_if_empty: bool = False) -> None:
        _ = keep_if_empty
        return

    def step(self, _description: str) -> None:
        return


@dataclass(frozen=True)
class PassResult:
    changed: bool = False

    @staticmethod
    def no_change() -> "PassResult":
        return PassResult(changed=False)

    @staticmethod
    def changed_result() -> "PassResult":
        return PassResult(changed=True)

    def merge(self, other: "PassResult") -> "PassResult":
        return PassResult(changed=self.changed or other.changed)


@dataclass
class TransformContext:
    program: UdonProgramData
    ir_class: Optional[IRClass] = None
    settings: Dict[str, Any] = field(default_factory=dict)
    metadata: Dict[str, Any] = field(default_factory=dict)
    stepper: TransformStepper = field(default_factory=TransformStepper)
    cancellation_check: Optional[Callable[[], None]] = None

    def check_cancellation(self) -> None:
        if self.cancellation_check is not None:
            self.cancellation_check()

    def fork_for_function(self, function: IRFunction) -> "FunctionTransformContext":
        return FunctionTransformContext(self, function)


class FunctionTransformContext(TransformContext):
    def __init__(self, parent: TransformContext, function: IRFunction):
        super().__init__(
            program=parent.program,
            ir_class=parent.ir_class,
            settings=parent.settings,
            metadata=parent.metadata,
            stepper=parent.stepper,
            cancellation_check=parent.cancellation_check,
        )
        self.parent = parent
        self.function = function


class BlockTransformContext(FunctionTransformContext):
    def __init__(
        self,
        parent: FunctionTransformContext,
        container: IRBlockContainer,
        control_flow_graph: nx.DiGraph,
        immediate_dominators: Dict[IRBlock, IRBlock],
        dominator_tree_children: Dict[IRBlock, List[IRBlock]],
        block: Optional[IRBlock] = None,
    ):
        super().__init__(parent, parent.function)
        self.parent = parent
        self.container = container
        self.control_flow_graph = control_flow_graph
        self.immediate_dominators = immediate_dominators
        self.dominator_tree_children = dominator_tree_children
        self.block = block


class StatementTransformContext(BlockTransformContext):
    def __init__(self, block_context: BlockTransformContext):
        super().__init__(
            parent=block_context,
            container=block_context.container,
            control_flow_graph=block_context.control_flow_graph,
            immediate_dominators=block_context.immediate_dominators,
            dominator_tree_children=block_context.dominator_tree_children,
            block=block_context.block,
        )
        self.block_context = block_context
        self._rerun_current_position = False
        self._rerun_position: Optional[int] = None

    def request_rerun(self, pos: Optional[int] = None) -> None:
        if pos is None:
            self._rerun_current_position = True
            return
        if self._rerun_position is None or pos > self._rerun_position:
            self._rerun_position = pos


class TransformPass(ABC):
    name: Optional[str] = None

    @property
    def display_name(self) -> str:
        return self.name or self.__class__.__name__

    def __str__(self) -> str:
        return self.display_name


class ProgramPass(TransformPass):
    @abstractmethod
    def run(self, functions: List[IRFunction], ctx: TransformContext) -> PassResult:
        raise NotImplementedError


class FunctionPass(TransformPass):
    @abstractmethod
    def run(self, function: IRFunction, ctx: FunctionTransformContext) -> PassResult:
        raise NotImplementedError


class BlockPass(TransformPass):
    @abstractmethod
    def run(self, block: IRBlock, ctx: BlockTransformContext) -> PassResult:
        raise NotImplementedError


class StatementPass(TransformPass):
    @abstractmethod
    def run(
        self, block: IRBlock, pos: int, ctx: StatementTransformContext
    ) -> PassResult:
        raise NotImplementedError


class LoopingBlockTransform(BlockPass):
    """
    Re-run child block transforms until a fixed point is reached.
    Mirrors ILSpy's LoopingBlockTransform behavior.
    """

    def __init__(self, *children: BlockPass, max_iterations: int = 32):
        self.children: Sequence[BlockPass] = children
        self.max_iterations = max_iterations
        self._running = False

    def run(self, block: IRBlock, ctx: BlockTransformContext) -> PassResult:
        if self._running:
            raise RuntimeError(
                "LoopingBlockTransform is already running; reentrancy is not supported."
            )
        self._running = True
        try:
            any_change = False
            for _ in range(self.max_iterations):
                changed = False
                for child in self.children:
                    ctx.check_cancellation()
                    child_result = child.run(block, ctx)
                    changed = changed or child_result.changed
                any_change = any_change or changed
                if not changed:
                    return PassResult(changed=any_change)
            raise RuntimeError(
                "LoopingBlockTransform did not reach a fixed point "
                f"within {self.max_iterations} iterations."
            )
        finally:
            self._running = False


class StatementTransform(BlockPass):
    """
    Runs statement transforms right-to-left with rerun support, similar to ILSpy.
    """

    def __init__(self, *children: StatementPass):
        self.children: Sequence[StatementPass] = children

    def run(self, block: IRBlock, ctx: BlockTransformContext) -> PassResult:
        if not block.statements:
            return PassResult.no_change()

        stmt_ctx = StatementTransformContext(ctx)
        changed = False
        pos = 0
        stmt_ctx._rerun_position = len(block.statements) - 1
        while pos >= 0:
            ctx.check_cancellation()
            if not block.statements:
                break

            if stmt_ctx._rerun_position is not None:
                next_pos = min(stmt_ctx._rerun_position, len(block.statements) - 1)
                pos = next_pos
                stmt_ctx._rerun_position = None
            elif pos >= len(block.statements):
                pos = len(block.statements) - 1

            for transform in self.children:
                result = transform.run(block, pos, stmt_ctx)
                changed = changed or result.changed

                if stmt_ctx._rerun_current_position:
                    stmt_ctx._rerun_current_position = False
                    stmt_ctx.request_rerun(pos)
                if stmt_ctx._rerun_position is not None:
                    break

            if stmt_ctx._rerun_position is None:
                pos -= 1

        return PassResult(changed=changed)


class BlockILTransform(FunctionPass):
    """
    Function transform that visits blocks in dominator-tree order.

    - Pre-order transforms run before visiting dominated children.
    - Post-order transforms run after dominated children are processed.
    """

    def __init__(
        self,
        pre_order_transforms: Optional[Iterable[BlockPass]] = None,
        post_order_transforms: Optional[Iterable[BlockPass]] = None,
    ):
        self.pre_order_transforms = list(pre_order_transforms or [])
        self.post_order_transforms = list(post_order_transforms or [])
        self._running = False

    def run(self, function: IRFunction, ctx: FunctionTransformContext) -> PassResult:
        if self._running:
            raise RuntimeError("BlockILTransform reentrancy detected.")
        self._running = True
        try:
            any_change = False
            for container in iter_block_containers(function):
                ctx.check_cancellation()
                if container.entry_block is None:
                    continue

                cfg = build_container_cfg(container)
                entry = container.entry_block
                if entry not in cfg:
                    continue

                try:
                    immediate_dominators = nx.immediate_dominators(cfg, entry)
                except Exception:
                    logger.debug(
                        "Failed to compute dominators for container in %s",
                        function.function_name,
                    )
                    continue

                dom_children = build_dominator_tree_children(
                    immediate_dominators, entry
                )

                block_ctx = BlockTransformContext(
                    parent=ctx,
                    container=container,
                    control_flow_graph=cfg,
                    immediate_dominators=immediate_dominators,
                    dominator_tree_children=dom_children,
                )

                def visit(block: IRBlock) -> None:
                    nonlocal any_change
                    block_ctx.block = block
                    for transform in self.pre_order_transforms:
                        block_ctx.check_cancellation()
                        result = transform.run(block, block_ctx)
                        any_change = any_change or result.changed

                    for child in dom_children.get(block, []):
                        visit(child)

                    block_ctx.block = block
                    for transform in self.post_order_transforms:
                        block_ctx.check_cancellation()
                        result = transform.run(block, block_ctx)
                        any_change = any_change or result.changed

                visit(entry)

            return PassResult(changed=any_change)
        finally:
            self._running = False
