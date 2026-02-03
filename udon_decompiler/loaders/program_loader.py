import json
from pathlib import Path
from typing import Any, Dict

from udon_decompiler.models import (
    EntryPointInfo,
    HeapEntry,
    HeapEntryValue,
    SymbolInfo,
    UdonProgramData,
)
from udon_decompiler.models.instruction import OpCode
from udon_decompiler.utils import logger


class ProgramLoader:
    @staticmethod
    def load_from_file(file_path: str | Path) -> UdonProgramData:
        file_path = Path(file_path)
        logger.debug(f"Loading Udon program from: {file_path}")

        if not file_path.exists():
            raise FileNotFoundError(f"Program file not found: {file_path}")

        with open(file_path, "r", encoding="utf-8") as f:
            data = json.load(f)

        return ProgramLoader._parse_program_data(data)

    @staticmethod
    def load_from_json_string(json_str: str) -> UdonProgramData:
        data = json.loads(json_str)
        return ProgramLoader._parse_program_data(data)

    @staticmethod
    def _parse_program_data(data: Dict[str, Any]) -> UdonProgramData:
        try:
            symbols = {}
            if "symbols" in data:
                for name, symbol_data in data["symbols"].items():
                    symbols[name] = SymbolInfo(
                        name=symbol_data["name"],
                        type=symbol_data["type"],
                        address=symbol_data["address"],
                    )

            entry_points = []
            if "entryPoints" in data:
                for ep_data in data["entryPoints"]:
                    entry_points.append(
                        EntryPointInfo(
                            name=ep_data["name"],
                            address=ep_data["address"],
                            # will be fixed after disassembled
                            call_jump_target=ep_data["address"],
                        )
                    )

            heap_initial_values = {}
            if "heapInitialValues" in data:
                for addr_str, heap_data in data["heapInitialValues"].items():
                    address = int(addr_str)
                    heap_initial_values[address] = HeapEntry(
                        address=heap_data["address"],
                        type=heap_data["type"],
                        value=HeapEntryValue(
                            is_serializable=heap_data["value"]["isSerializable"],
                            value=heap_data["value"]["value"],
                        ),
                    )

            program = UdonProgramData(
                byte_code_hex=data["byteCodeHex"],
                byte_code_length=data["byteCodeLength"],
                symbols=symbols,
                entry_points=entry_points,
                heap_initial_values=heap_initial_values,
            )

            logger.info(f"Successfully loaded program: {program}")
            return program

        except KeyError as e:
            logger.error(f"Missing required field in JSON: {e}")
            raise KeyError(f"Invalid program JSON format: missing field {e}")
