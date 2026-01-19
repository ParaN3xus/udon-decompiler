from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional

from udon_decompiler.utils import logger


@dataclass
class SymbolInfo:
    name: str
    type: str
    address: int

    def __repr__(self) -> str:
        return f"Symbol({self.name}: {self.type} @ 0x{self.address:08x})"


@dataclass
class HeapEntryValue:
    is_serializable: bool
    value: Any

    def __repr__(self) -> str:
        if self.is_serializable:
            return f"HeapValue({repr(self.value)})"
        return "HeapValue(<non-serializable>)"


@dataclass
class HeapEntry:
    address: int
    type: str
    value: HeapEntryValue

    def __repr__(self) -> str:
        return f"Heap[0x{self.address:08x}]: {self.type} = {self.value}"


@dataclass
class EntryPointInfo:
    name: str
    address: int

    def __repr__(self) -> str:
        return f"EntryPoint({self.name} @ 0x{self.address:08x})"


@dataclass
class UdonProgramData:
    byte_code_hex: str
    byte_code_length: int
    symbols: Dict[str, SymbolInfo] = field(default_factory=dict)
    entry_points: List[EntryPointInfo] = field(default_factory=list)
    heap_initial_values: Dict[int, HeapEntry] = field(default_factory=dict)

    CLASS_NAME_ADDR: int = 1
    CLASS_NAME_SYMBOL_NAME: str = "__refl_typename"

    def get_symbol_by_address(self, address: int) -> Optional[SymbolInfo]:
        for symbol in self.symbols.values():
            if symbol.address == address:
                return symbol
        return None

    def get_entry_point_by_address(self, address: int) -> Optional[EntryPointInfo]:
        for entry_point in self.entry_points:
            if entry_point.address == address:
                return entry_point
        return None

    def get_initial_heap_value(self, address: int) -> Optional[HeapEntry]:
        return self.heap_initial_values.get(address)

    def get_class_name(self) -> Optional[str]:
        possible_class_name_symbol = self.get_symbol_by_address(
            UdonProgramData.CLASS_NAME_ADDR
        )
        if not possible_class_name_symbol:
            logger.warning(
                "Class name symbol not found! The Udon program might be broken!"
            )
            return None
        if possible_class_name_symbol.name != UdonProgramData.CLASS_NAME_SYMBOL_NAME:
            logger.warning(
                "Incorrect class name symbol found! The Udon program might be broken!"
            )
            return None

        possible_class_name_entry = self.get_initial_heap_value(
            UdonProgramData.CLASS_NAME_ADDR
        )
        if not possible_class_name_entry:
            logger.warning(
                "Class name entry not found! The Udon program might be broken!"
            )
            return None
        if not possible_class_name_entry.value.is_serializable:
            logger.warning(
                "Class name entry is not serializable! The Udon program might be broken!"
            )
            return None

        # todo: type verification
        return possible_class_name_entry.value.value

    @property
    def byte_code_bytes(self) -> bytes:
        return bytes.fromhex(self.byte_code_hex)

    def __repr__(self) -> str:
        return (
            f"UdonProgram(\n"
            f"  bytecode_length={self.byte_code_length},\n"
            f"  symbols={len(self.symbols)},\n"
            f"  entry_points={len(self.entry_points)},\n"
            f"  heap_entries={len(self.heap_initial_values)}\n"
            f")"
        )
