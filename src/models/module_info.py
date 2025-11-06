from dataclasses import dataclass, field
from typing import Dict, Optional, Tuple


@dataclass
class ExternFunctionInfo:
    signature: str
    parameter_count: int
    module_name: str
    function_name: str

    def __repr__(self) -> str:
        return f"Extern({self.signature}, params={self.parameter_count})"


class Singleton(type):
    _instances = {}

    def __call__(cls, *args, **kwargs):
        if cls not in cls._instances:
            cls._instances[cls] = super().__call__(*args, **kwargs)
        return cls._instances[cls]


@dataclass
class UdonModuleInfo(metaclass=Singleton):
    modules: Dict[str, Dict[str, int]] = field(default_factory=dict)
    # modules[module_name][func_signature] = parameter_count

    def get_function_info(self, signature: str) -> Optional[ExternFunctionInfo]:
        (module_name, func_name) = self.parse_signature(signature)

        return ExternFunctionInfo(
            signature=signature,
            parameter_count=self.modules[module_name][func_name],
            module_name=module_name,
            function_name=func_name
        )

    @staticmethod
    def parse_signature(signature: str) -> Tuple[str, str]:
        parts = signature.split(".", 2)
        module_name = parts[0]
        func_name = parts[1]
        return (module_name, func_name)

    def add_module(self, module_name: str, functions: Dict[str, int]) -> None:
        self.modules[module_name] = functions

    def get_parameter_count(self, signature: str) -> Optional[int]:
        info = self.get_function_info(signature)
        return info.parameter_count if info else None

    def __repr__(self) -> str:
        total_functions = sum(len(funcs) for funcs in self.modules.values())
        return (
            f"UdonModuleInfo(\n"
            f"  modules={len(self.modules)},\n"
            f"  total_functions={total_functions}\n"
            f")"
        )
