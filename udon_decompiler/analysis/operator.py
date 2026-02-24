from enum import Enum


class Operator(Enum):
    formatter: str

    Addition = ("Addition", "{} + {}")
    Subtraction = ("Subtraction", "{} - {}")
    Multiplication = (("Multiplication", "Multiply"), "{} * {}")
    Division = ("Division", "{} / {}")
    Remainder = ("Remainder", "{} % {}")
    UnaryMinus = ("UnaryMinus", "-{}")
    UnaryNegation = ("UnaryNegation", "!{}")
    ConditionalAnd = ("ConditionalAnd", "{} && {}")
    ConditionalOr = ("ConditionalOr", "{} || {}")
    ConditionalXor = ("ConditionalXor", "{} ^ {}")
    LogicalAnd = ("LogicalAnd", "{} && {}")
    LogicalOr = ("LogicalOr", "{} || {}")
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
