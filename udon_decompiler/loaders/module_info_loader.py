import json
from pathlib import Path
from typing import Any, Dict

from udon_decompiler.models import UdonModuleInfo
from udon_decompiler.utils import logger


class ModuleInfoLoader:
    @staticmethod
    def load_from_file(file_path: str | Path) -> UdonModuleInfo:
        file_path = Path(file_path)
        logger.debug(f"Loading module info from: {file_path}")

        if not file_path.exists():
            raise FileNotFoundError(f"Module info file not found: {file_path}")

        with open(file_path, "r", encoding="utf-8") as f:
            data = json.load(f)

        return ModuleInfoLoader._parse_module_data(data)

    @staticmethod
    def load_from_json_string(json_str: str) -> UdonModuleInfo:
        data = json.loads(json_str)
        return ModuleInfoLoader._parse_module_data(data)

    @staticmethod
    def _parse_module_data(data: Dict[str, Dict[str, Any]]) -> UdonModuleInfo:
        module_info = UdonModuleInfo()

        for module_name, functions in data.items():
            module_info.add_module(module_name, functions)

        logger.info(f"Successfully loaded module info")
        return module_info
