from udon_decompiler.analysis.ir.nodes import IRFunction
from udon_decompiler.analysis.transform.pass_base import (
    FunctionPass,
    PassResult,
    ProgramPass,
    TransformContext,
)
from udon_decompiler.utils.logger import logger


class TransformPipeline:
    def __init__(
        self,
        passes: list[FunctionPass | ProgramPass],
    ):
        self.passes = passes

    def run(
        self,
        functions: list[IRFunction],
        ctx: TransformContext,
    ) -> None:
        for p in self.passes:
            if isinstance(p, FunctionPass):
                for function in functions:
                    result = p.run(function, ctx)
                    if result.changed:
                        logger.debug(
                            f"Pass '{p.name}' changed '{function.function_name}'"
                        )
            elif isinstance(p, ProgramPass):
                result = p.run(functions, ctx)
                if result.changed:
                    logger.debug(f"Program pass '{p.name}' made changes")


def build_default_pipeline() -> TransformPipeline:
    from udon_decompiler.analysis.transform.passes.class_construction import (
        IRClassConstructionPass,
    )

    return TransformPipeline(
        passes=[
            # ConstToLiteralPass(),
            # TempVariableInlinePass(),
            # structure ...
            # CollectVariablesPass(),
            # IRClassConstructionPass(),
            # PromoteGlobalsPass(),
        ]
    )
