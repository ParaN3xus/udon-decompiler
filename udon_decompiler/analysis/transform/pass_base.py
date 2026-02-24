from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any, Callable, Dict, Iterable, List, Optional, Sequence, cast

from udon_decompiler.analysis.ir.control_flow_graph import ControlFlowGraph
from udon_decompiler.analysis.ir.control_flow_node import ControlFlowNode
from udon_decompiler.analysis.ir.nodes import (
    IRBlock,
    IRClass,
    IRFunction,
)
from udon_decompiler.analysis.transform.ir_utils import iter_block_containers
from udon_decompiler.models.program import UdonProgramData


class TransformStepper:
    """No-op stepper"""

    def start_group(self, _description: str) -> None:
        return

    def end_group(self, keep_if_empty: bool = False) -> None:
        _ = keep_if_empty
        return

    def step(self, _description: str) -> None:
        return


@dataclass
class ProgramTransformContext:
    """
    Program-level transform context.
    """

    program: UdonProgramData
    ir_class: Optional[IRClass] = None
    settings: Dict[str, Any] = field(default_factory=dict)
    metadata: Dict[str, Any] = field(default_factory=dict)
    stepper: TransformStepper = field(default_factory=TransformStepper)
    cancellation_check: Optional[Callable[[], None]] = None

    def throw_if_cancellation_requested(self) -> None:
        if self.cancellation_check is not None:
            self.cancellation_check()

    def create_il_context(self, function: IRFunction) -> "TransformContext":
        return TransformContext(function=function, program_context=self)


class TransformContext:
    """Per-function context for ITransform"""

    def __init__(
        self,
        function: IRFunction,
        program_context: ProgramTransformContext,
    ):
        self.function = function
        self.program_context = program_context

    @property
    def program(self) -> UdonProgramData:
        return self.program_context.program

    @property
    def ir_class(self) -> Optional[IRClass]:
        return self.program_context.ir_class

    @ir_class.setter
    def ir_class(self, value: Optional[IRClass]) -> None:
        self.program_context.ir_class = value

    @property
    def settings(self) -> Dict[str, Any]:
        return self.program_context.settings

    @property
    def metadata(self) -> Dict[str, Any]:
        return self.program_context.metadata

    @property
    def stepper(self) -> TransformStepper:
        return self.program_context.stepper

    def throw_if_cancellation_requested(self) -> None:
        self.program_context.throw_if_cancellation_requested()

    def step_start_group(self, description: str) -> None:
        self.stepper.start_group(description)

    def step_end_group(self, keep_if_empty: bool = False) -> None:
        self.stepper.end_group(keep_if_empty=keep_if_empty)

    def step(self, description: str) -> None:
        self.stepper.step(description)


class IProgramTransform(ABC):
    """Program-level transform that sees all functions."""

    name: Optional[str] = None

    @property
    def display_name(self) -> str:
        return self.name or self.__class__.__name__

    @abstractmethod
    def run(
        self,
        functions: List[IRFunction],
        context: ProgramTransformContext,
    ) -> None:
        raise NotImplementedError


class ITransform(ABC):
    """Per-function transform."""

    name: Optional[str] = None

    @property
    def display_name(self) -> str:
        return self.name or self.__class__.__name__

    @abstractmethod
    def run(self, function: IRFunction, context: TransformContext) -> None:
        raise NotImplementedError


class IBlockTransform(ABC):
    """Per-block transform, usually orchestrated by BlockTransform."""

    @abstractmethod
    def run(self, block: IRBlock, context: "BlockTransformContext") -> None:
        raise NotImplementedError


class IStatementTransform(ABC):
    """Per-statement transform, executed right-to-left inside one block."""

    @abstractmethod
    def run(
        self,
        block: IRBlock,
        pos: int,
        context: "StatementTransformContext",
    ) -> None:
        raise NotImplementedError


class BlockTransformContext(TransformContext):
    def __init__(
        self,
        context: TransformContext,
        control_flow_graph: ControlFlowGraph,
    ):
        super().__init__(
            function=context.function,
            program_context=context.program_context,
        )
        self.container = control_flow_graph.container
        self.control_flow_graph = control_flow_graph

        self.block: Optional[IRBlock] = None
        self.control_flow_node: Optional[ControlFlowNode] = None
        self._dirty = False

    @property
    def is_dirty(self) -> bool:
        return self._dirty

    def mark_dirty(self) -> None:
        self._dirty = True

    def reset_dirty(self) -> None:
        self._dirty = False


class StatementTransformContext(TransformContext):
    def __init__(self, block_context: BlockTransformContext):
        super().__init__(
            function=block_context.function,
            program_context=block_context.program_context,
        )
        self.block_context = block_context

        self.rerun_current_position = False
        self.rerun_position: Optional[int] = None

    @property
    def block(self) -> Optional[IRBlock]:
        return self.block_context.block

    def request_rerun(self, pos: Optional[int] = None) -> None:
        if pos is None:
            self.rerun_current_position = True
            return
        if self.rerun_position is None or pos > self.rerun_position:
            self.rerun_position = pos


def run_block_transforms(
    block: IRBlock,
    transforms: Iterable[IBlockTransform],
    context: BlockTransformContext,
) -> None:
    for transform in transforms:
        context.throw_if_cancellation_requested()
        context.step_start_group(transform.__class__.__name__)
        transform.run(block, context)
        context.step_end_group()


def run_il_transforms(
    function: IRFunction,
    transforms: Iterable[ITransform],
    context: TransformContext,
) -> None:
    for transform in transforms:
        context.throw_if_cancellation_requested()
        context.step_start_group(transform.display_name)
        transform.run(function, context)
        context.step_end_group(keep_if_empty=True)


class LoopingBlockTransform(IBlockTransform):
    """
    Repeats child block transforms until the block no longer changes.
    """

    def __init__(self, *transforms: IBlockTransform):
        self.transforms: Sequence[IBlockTransform] = transforms
        self._running = False

    def run(self, block: IRBlock, context: BlockTransformContext) -> None:
        if self._running:
            raise RuntimeError(
                "LoopingBlockTransform already running. Transforms are not re-entrant."
            )

        self._running = True
        try:
            count = 1
            while True:
                context.reset_dirty()
                run_block_transforms(block, self.transforms, context)
                if not context.is_dirty:
                    break
                count += 1
                context.step(f"Block is dirty; running loop iteration #{count}.")
        finally:
            self._running = False


class StatementTransform(IBlockTransform):
    """
    Runs statement transforms with right-to-left + rerun semantics.
    """

    def __init__(self, *children: IStatementTransform):
        self.children: Sequence[IStatementTransform] = children

    def run(self, block: IRBlock, context: BlockTransformContext) -> None:
        if not block.statements:
            return

        stmt_ctx = StatementTransformContext(context)

        pos = 0
        stmt_ctx.rerun_position = len(block.statements) - 1
        while pos >= 0:
            context.throw_if_cancellation_requested()
            if not block.statements:
                break

            if stmt_ctx.rerun_position is not None:
                pos = min(stmt_ctx.rerun_position, len(block.statements) - 1)
                stmt_ctx.rerun_position = None
            elif pos >= len(block.statements):
                pos = len(block.statements) - 1

            for transform in self.children:
                transform.run(block, pos, stmt_ctx)

                if stmt_ctx.rerun_current_position:
                    stmt_ctx.rerun_current_position = False
                    stmt_ctx.request_rerun(pos)
                if stmt_ctx.rerun_position is not None:
                    break

            if stmt_ctx.rerun_position is None:
                pos -= 1


class BlockTransform(ITransform):
    """
    Transform that runs block transforms in dominator-tree order.

    Pre-order transforms run before dominated children;
    post-order transforms run after children.
    """

    def __init__(self):
        self.pre_order_transforms: List[IBlockTransform] = []
        self.post_order_transforms: List[IBlockTransform] = []
        self._running = False

    def run(self, function: IRFunction, context: TransformContext) -> None:
        if self._running:
            raise RuntimeError(
                "Reentrancy detected. Transforms are not thread-safe/re-entrant."
            )

        self._running = True
        try:
            # Snapshot containers before transforms
            # This avoids re-processing newly created containers
            # in the same BlockTransform run (e.g. during loop/switch detection).
            containers = list(iter_block_containers(function))
            for container in containers:
                context.throw_if_cancellation_requested()
                if container.entry_block is None:
                    continue

                cfg = ControlFlowGraph(
                    container=container,
                    function_body=function.body,
                )
                entry_node = cfg.get_node(container.entry_block)

                block_ctx = BlockTransformContext(
                    context=context,
                    control_flow_graph=cfg,
                )

                self._visit_block(entry_node, block_ctx)
        finally:
            self._running = False

    def _visit_block(
        self,
        cfg_node: ControlFlowNode,
        context: BlockTransformContext,
    ) -> None:
        block = cfg_node.block
        if block is None:
            raise RuntimeError("ControlFlowNode.block is None for non-sentinel node")
        context.block = block
        context.control_flow_node = cfg_node
        context.step_start_group(block.label)

        run_block_transforms(block, self.pre_order_transforms, context)

        for child in cfg_node.dominator_tree_children or []:
            self._visit_block(child, context)

        context.block = block
        context.control_flow_node = cfg_node
        run_block_transforms(block, self.post_order_transforms, context)

        context.step_end_group()
