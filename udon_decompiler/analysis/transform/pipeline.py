from __future__ import annotations

from typing import Iterable, List, Sequence

from udon_decompiler.analysis.ir.nodes import IRFunction
from udon_decompiler.analysis.transform.pass_base import (
    IILTransform,
    IProgramTransform,
    ProgramTransformContext,
    run_il_transforms,
)


class TransformPipeline:
    """
    ILSpy-style transform pipeline:
    - run all IILTransform on each function
    - then run program-level transforms
    """

    def __init__(
        self,
        il_transforms: Iterable[IILTransform] = (),
        program_transforms: Iterable[IProgramTransform] = (),
    ):
        self.il_transforms: List[IILTransform] = list(il_transforms)
        self.program_transforms: List[IProgramTransform] = list(program_transforms)

    def run(
        self,
        functions: Sequence[IRFunction],
        context: ProgramTransformContext,
    ) -> None:
        function_list = list(functions)

        for function in function_list:
            context.throw_if_cancellation_requested()
            il_context = context.create_il_context(function)
            run_il_transforms(function, self.il_transforms, il_context)

        for transform in self.program_transforms:
            context.throw_if_cancellation_requested()
            context.stepper.start_group(transform.display_name)
            transform.run(function_list, context)
            context.stepper.end_group(keep_if_empty=True)


def build_default_pipeline() -> TransformPipeline:
    from udon_decompiler.analysis.transform.pass_base import BlockILTransform
    from udon_decompiler.analysis.transform.passes.class_construction import (
        IRClassConstructionTransform,
    )
    from udon_decompiler.analysis.transform.passes.condition_detection import (
        ConditionDetection,
    )
    from udon_decompiler.analysis.transform.passes.detect_exit_points import (
        DetectExitPoints,
    )
    from udon_decompiler.analysis.transform.passes.loop_detection import LoopDetection

    block_loop_transform = BlockILTransform()
    block_loop_transform.post_order_transforms.append(LoopDetection())

    block_condition_transform = BlockILTransform()
    block_condition_transform.post_order_transforms.append(ConditionDetection())

    return TransformPipeline(
        il_transforms=[
            DetectExitPoints(can_introduce_exit_for_return=False),
            block_loop_transform,
            DetectExitPoints(can_introduce_exit_for_return=True),
            block_condition_transform,
        ],
        program_transforms=[
            IRClassConstructionTransform(),
        ],
    )
