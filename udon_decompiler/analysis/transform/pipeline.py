from __future__ import annotations

from typing import Iterable, List, Sequence

from udon_decompiler.analysis.ir.nodes import IRFunction
from udon_decompiler.analysis.transform.pass_base import (
    FunctionPass,
    FunctionTransformContext,
    PassResult,
    ProgramPass,
    TransformContext,
)
from udon_decompiler.utils.logger import logger


class TransformPipeline:
    """
    Pipeline runner for mixed ProgramPass / FunctionPass transforms.
    """

    def __init__(self, passes: Iterable[ProgramPass | FunctionPass]):
        self.passes: List[ProgramPass | FunctionPass] = list(passes)

    def run(self, functions: Sequence[IRFunction], ctx: TransformContext) -> PassResult:
        function_list = list(functions)
        aggregate = PassResult.no_change()

        for transform in self.passes:
            ctx.check_cancellation()
            group_name = transform.display_name
            ctx.stepper.start_group(group_name)
            logger.debug("Running transform: %s", group_name)

            if isinstance(transform, ProgramPass):
                result = transform.run(function_list, ctx)
            elif isinstance(transform, FunctionPass):
                result = self._run_function_pass(transform, function_list, ctx)
            else:
                raise TypeError(
                    "Unsupported transform type: "
                    f"{type(transform).__name__}. "
                    "Expected ProgramPass or FunctionPass."
                )

            aggregate = aggregate.merge(result)
            ctx.stepper.end_group(keep_if_empty=True)

        return aggregate

    @staticmethod
    def _run_function_pass(
        transform: FunctionPass,
        functions: Sequence[IRFunction],
        ctx: TransformContext,
    ) -> PassResult:
        changed = False
        for function in functions:
            ctx.check_cancellation()
            fn_ctx = FunctionTransformContext(ctx, function)
            result = transform.run(function, fn_ctx)
            changed = changed or result.changed
        return PassResult(changed=changed)


def build_default_pipeline() -> TransformPipeline:
    from udon_decompiler.analysis.transform.passes.class_construction import (
        IRClassConstructionPass,
    )

    # Keep default minimal and always valid.
    # More transforms are intentionally added explicitly by callers.
    return TransformPipeline(
        passes=[
            IRClassConstructionPass(),
        ]
    )
