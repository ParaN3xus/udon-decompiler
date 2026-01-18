from dataclasses import dataclass, field
from typing import Dict, Optional, Tuple, List, Any


class Singleton(type):
    _instances = {}

    def __call__(cls, *args, **kwargs):
        if cls not in cls._instances:
            cls._instances[cls] = super().__call__(*args, **kwargs)
        return cls._instances[cls]


@dataclass
class ExternFunctionInfo:
    signature: str
    parameter_count: int
    module_name: str
    function_name: str
    def_type: str
    is_static: Optional[bool] = None
    original_name: Optional[str] = None

    def __repr__(self) -> str:
        extras = []
        if self.is_static is not None:
            extras.append(f"static={self.is_static}")
        if self.original_name:
            extras.append(f"orig={self.original_name}")

        extras_str = ", ".join(extras)
        if extras_str:
            extras_str = ", " + extras_str

        return (f"Extern({self.signature}, params={self.parameter_count}, "
                f"type={self.def_type}{extras_str})")


@dataclass
class FunctionMetadata:
    parameter_count: int
    def_type: str
    is_static: Optional[bool] = None
    original_name: Optional[str] = None


@dataclass
class ModuleMetadata:
    type_name: str
    functions: Dict[str, FunctionMetadata] = field(default_factory=dict)


@dataclass
class UdonModuleInfo(metaclass=Singleton):
    # modules[module_name] = ModuleMetadata
    modules: Dict[str, ModuleMetadata] = field(default_factory=dict)

    def get_function_info(self, signature: str) -> Optional[ExternFunctionInfo]:
        try:
            (module_name, func_name) = self.parse_signature(signature)

            module_meta = self.modules.get(module_name)
            if not module_meta:
                return None

            func_meta = module_meta.functions.get(func_name)
            if not func_meta:
                return None

            return ExternFunctionInfo(
                signature=signature,
                parameter_count=func_meta.parameter_count,
                module_name=module_name,
                function_name=func_name,
                def_type=func_meta.def_type,
                is_static=func_meta.is_static,
                original_name=func_meta.original_name
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
            name = func_data['name']

            meta = FunctionMetadata(
                parameter_count=func_data['parameterCount'],
                def_type=func_data['defType'],
                is_static=func_data.get('isStatic'),
                original_name=func_data.get('originalName')
            )
            func_dict[name] = meta

        self.modules[module_name] = ModuleMetadata(
            type_name=type_name,
            functions=func_dict
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
