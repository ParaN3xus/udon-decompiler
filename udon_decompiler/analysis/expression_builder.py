from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Final, List, Optional

from udon_decompiler.analysis.stack_simulator import (
    StackSimulator,
    StackValue,
)
from udon_decompiler.analysis.variable_identifier import VariableIdentifier
from udon_decompiler.models.instruction import Instruction, OpCode
from udon_decompiler.models.module_info import (
    ExternFunctionInfo,
    FunctionDefinitionType,
    UdonModuleInfo,
)
from udon_decompiler.models.program import EntryPointInfo, SymbolInfo, UdonProgramData
from udon_decompiler.utils.logger import logger


class ExpressionType(Enum):
    LITERAL = "literal"
    VARIABLE = "variable"  # var ref
    OPERATOR = "op"
    EXTERNAL_CALL = "ext_call"
    INTERNAL_CALL = "int_call"
    PROPERTY_ACCESS = "prop"
    CONSTRUCTOR = "ctor"
    # ARRAY_ACCESS = "array"        # arr access
    ASSIGNMENT = "assignment"
    # CAST = "cast"                 # cast


class Operator(Enum):
    formatter: str

    Addition = ("Addition", "{} + {}")
    Subtraction = ("Subtraction", "{} - {}")
    Multiplication = (("Multiplication", "Multiply"), "{} * {}")
    Division = ("Division", "{} / {}")
    Remainder = ("Remainder", "{} % {}")
    UnaryMinus = ("UnaryMinus", "-{}")
    UnaryNegation = ("UnaryNegation", "!{}")
    LogicalAnd = ("LogicalAnd", "{} & {}")
    LogicalOr = ("LogicalOr", "{} | {}")
    LogicalXor = ("LogicalXor", "{} ^ {}")
    LeftShift = ("LeftShift", "{} << {}")
    RightShift = ("RightShift", "{} >> {}")
    Equality = ("Equality", "{} == {}")
    Inequality = ("Inequality", "{} != {}")
    GreaterThan = ("GreaterThan", "{} > {}")
    GreaterThanOrEqual = ("GreaterThanOrEqual", "{} >= {}")
    LessThan = ("LessThan", "{} < {}")
    LessThanOrEqual = ("LessThanOrEqual", "{} <= {}")
    Conversion = (("Implicit", "Explicit"), "({}){}")

    def __new__(cls, names, formatter):
        obj = object.__new__(cls)

        if isinstance(names, str):
            names = (names,)

        obj._value_ = names
        obj.formatter = formatter
        return obj

    @classmethod
    def from_name(cls, name: str):
        if not hasattr(cls, "_lookup_cache"):
            cls._lookup_cache = {}
            for member in cls:
                for alias in member.value:
                    cls._lookup_cache[alias] = member

        res = cls._lookup_cache.get(name)
        if res is None:
            raise Exception(f"Invalid operator: {name}")

        return res
