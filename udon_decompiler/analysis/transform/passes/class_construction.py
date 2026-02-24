from typing import Optional

from udon_decompiler.analysis.ir.nodes import IRClass, IRFunction
from udon_decompiler.analysis.transform.pass_base import (
    PassResult,
    ProgramPass,
    TransformContext,
)
from udon_decompiler.utils.logger import logger


class IRClassConstructionPass(ProgramPass):
    """Constructs the IRClass object and instantiates it in the context."""

    name = "ir-class-construction"

    _class_counter: int = 0

    def run(
        self,
        functions: list[IRFunction],
        ctx: TransformContext,
    ) -> PassResult:
        program = ctx.program
        class_name = program.get_class_name()
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

        ctx.ir_class = IRClass(
            class_name=class_name,
            namespace=namespace,
            program=program,
            variable_declarations=[],
            functions=functions,
        )

        return PassResult(changed=True)
