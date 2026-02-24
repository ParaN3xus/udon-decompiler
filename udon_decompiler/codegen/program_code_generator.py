from typing import Optional

from udon_decompiler.analysis.ir.nodes import IRClass
from udon_decompiler.codegen.code_generator import CSharpCodeGenerator
from udon_decompiler.utils.logger import logger


class ProgramCodeGenerator:
    _generator: CSharpCodeGenerator = CSharpCodeGenerator()

    @classmethod
    def generate_program(
        cls,
        class_ir: IRClass,
    ) -> tuple[Optional[str], str]:
        # We need to decide if we return class_name or None (if fallback).
        # Existing logic returned None if fallback.
        # For now let's just return the class name from IRClass.
        return class_ir.class_name, cls._generator.generate(class_ir)
