from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Optional

from udon_decompiler.analysis.ir.nodes import (
    IRClass,
    IRFunction,
)
from udon_decompiler.models.program import UdonProgramData


@dataclass
class PassResult:
    changed: bool


@dataclass
class TransformContext:
    program: UdonProgramData
    ir_class: Optional[IRClass] = None


class FunctionPass(ABC):
    """Runs on each IRFunction independently."""

    name: str

    @abstractmethod
    def run(
        self,
        function: IRFunction,
        ctx: TransformContext,
    ) -> PassResult: ...


class ProgramPass(ABC):
    """Runs on a list of IRFunctions (whole-program level)."""

    name: str

    @abstractmethod
    def run(
        self,
        functions: list[IRFunction],
        ctx: TransformContext,
    ) -> PassResult: ...
