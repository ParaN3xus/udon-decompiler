from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, Final, Optional, Tuple

from udon_decompiler.utils.utils import Singleton


class FunctionDefinitionType(Enum):
    METHOD = "method"
    FIELD = "prop"
    CTOR = "ctor"
    OPERATOR = "op"


class ParameterType(Enum):
    IN = "IN"
    OUT = "OUT"
    IN_OUT = "IN_OUT"


@dataclass
class ExternFunctionInfo:
    signature: str
    type_name: str
    function_name: str
    is_static: bool
    returns_void: bool
    def_type: FunctionDefinitionType
    parameters: list[ParameterType] = field(default_factory=list)
    original_name: Optional[str] = None

    @property
    def parameter_count(self) -> int:
        return len(self.parameters)

    def __repr__(self) -> str:
        extras = []
        if self.is_static is not None:
            extras.append(f"static={self.is_static}")
        if self.original_name:
            extras.append(f"orig={self.original_name}")

        extras_str = ", ".join(extras)
        if extras_str:
            extras_str = ", " + extras_str

        return (
            f"Extern({self.signature}, params={self.parameter_count}, "
            f"type={self.def_type}{extras_str})"
        )


@dataclass
class FunctionMetadata:
    def_type: FunctionDefinitionType
    is_static: bool
    returns_void: bool
    parameters: list[ParameterType] = field(default_factory=list)
    original_name: Optional[str] = None

    @property
    def parameter_count(self) -> int:
        return len(self.parameters)


@dataclass
class ModuleMetadata:
    type_name: str
    functions: Dict[str, FunctionMetadata] = field(default_factory=dict)


@dataclass
class UdonModuleInfo(metaclass=Singleton):
    UINT32ARRAY_GET_METHOD_NAME: Final[str] = (
        "SystemUInt32Array.__Get__SystemInt32__SystemUInt32"
    )
    UINT32_TYPE_NAME: Final[str] = "System.UInt32"
    ARRAY_TYPE_SUFFIX: Final[str] = "[]"

    # modules[module_name] = ModuleMetadata
    modules: Dict[str, ModuleMetadata] = field(default_factory=dict)

    @staticmethod
    def _parse_parameter_type(value: Any) -> ParameterType:
        if isinstance(value, ParameterType):
            return value
        if isinstance(value, str):
            try:
                return ParameterType[value]
            except KeyError:
                return ParameterType(value)
        raise ValueError(f"Unknown parameter type: {value}")

    def get_function_info(self, signature: str) -> Optional[ExternFunctionInfo]:
        try:
            (module_name, func_name) = self.parse_signature(signature)

            module_meta = self.modules.get(module_name)
            if module_meta is None:
                return None

            func_meta = module_meta.functions.get(func_name)
            if func_meta is None:
                return None

            return ExternFunctionInfo(
                signature=signature,
                parameters=func_meta.parameters,
                type_name=module_meta.type_name,
                function_name=func_name,
                def_type=func_meta.def_type,
                is_static=func_meta.is_static,
                returns_void=func_meta.returns_void,
                original_name=func_meta.original_name,
            )
        except Exception:
            return None

    def get_module_type(self, module_name: str) -> Optional[str]:
        module_meta = self.modules.get(module_name)
        return module_meta.type_name if module_meta else None

    @staticmethod
    def parse_signature(signature: str) -> Tuple[str, str]:
        parts = signature.split(".", 2)
        module_name = parts[0]
        func_name = parts[1]
        return (module_name, func_name)

    def add_module(self, module_name: str, module_data: Dict[str, Any]) -> None:
        type_name = module_data.get("type", "")
        functions_list = module_data.get("functions", [])

        func_dict = {}
        for func_data in functions_list:
            name = func_data["name"]
            raw_parameters = func_data.get("parameters", [])
            parameters = [self._parse_parameter_type(value) for value in raw_parameters]

            meta = FunctionMetadata(
                parameters=parameters,
                def_type=FunctionDefinitionType(func_data["defType"]),
                is_static=func_data.get("isStatic"),
                returns_void=func_data.get("returnsVoid"),
                original_name=func_data.get("originalName"),
            )
            func_dict[name] = meta

        self.modules[module_name] = ModuleMetadata(
            type_name=type_name, functions=func_dict
        )

    def get_parameter_count(self, signature: str) -> Optional[int]:
        info = self.get_function_info(signature)
        return info.parameter_count if info else None

    def __repr__(self) -> str:
        total_functions = sum(len(m.functions) for m in self.modules.values())
        return (
            f"UdonModuleInfo(\n"
            f"  modules={len(self.modules)},\n"
            f"  total_functions={total_functions}\n"
            f")"
        )
