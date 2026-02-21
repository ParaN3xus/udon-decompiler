from udon_decompiler.analysis.ir.nodes import IRClass, IRFunction
from udon_decompiler.analysis.transform.pass_base import (
    IProgramTransform,
    ProgramTransformContext,
)
from udon_decompiler.utils.logger import logger


class IRClassConstructionTransform(IProgramTransform):
    """Construct IRClass from analyzed functions."""

    name = "ir-class-construction"

    _class_counter: int = 0

    def run(
        self,
        functions: list[IRFunction],
        context: ProgramTransformContext,
    ) -> None:
        class_name = context.program.get_class_name()
        namespace = None

        if class_name is None:
            logger.warning(
                "Failed to identify class name, using synthetic fallback name."
            )
            self.__class__._class_counter += 1
            class_name = f"DecompiledClass_{self.__class__._class_counter}"
        elif "." in class_name:
            namespace, class_name = class_name.rsplit(".", 1)
            if not namespace:
                namespace = None

        context.ir_class = IRClass(
            class_name=class_name,
            namespace=namespace,
            program=context.program,
            variable_declarations=[],
            functions=functions,
        )
