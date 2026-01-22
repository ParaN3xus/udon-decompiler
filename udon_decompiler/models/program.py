from dataclasses import dataclass, field
from typing import Any, Dict, Final, List, Optional

from udon_decompiler.models.instruction import OpCode
from udon_decompiler.utils import logger


@dataclass
class SymbolInfo:
    name: str
    type: str
    address: int

    RETURN_JUMP_ADDR_SYMBOL_NAME: Final[str] = "__intnl_returnJump_SystemUInt32_0"
    HALT_JUMP_ADDR_SYMBOL_NAME: Final[str] = "__const_SystemUInt32_0"

    def __repr__(self) -> str:
        return f"Symbol({self.name}: {self.type} @ 0x{self.address:08x})"

    @staticmethod
    def try_parse_function_return(symbol_name: str) -> Optional[str]:
        # __{id1}___{id2}_{methodName}__ret
        if (
            len(symbol_name) < 14
            or not symbol_name.startswith("__")
            or not symbol_name.endswith("__ret")
        ):
            return None
        body = symbol_name[2:-5]
        if "___" not in body:
            return None
        id1_str, _, rest = body.partition("___")
        if "_" not in rest:
            return None
        id2_str, _, method_name = rest.partition("_")
        if id1_str.isdigit() and id2_str.isdigit() and method_name:
            return method_name
        return None


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
    """
    Information of entry points (functions)

    Attributes:
        name: The name of the function, can be None if undertermined
        address: The address of the `PUSH, __const_SystemUInt32_0` instruction
    """

    name: Optional[str]
    address: int

    @property
    def call_jump_target(self) -> int:
        return self.address + OpCode.PUSH.size

    def __repr__(self) -> str:
        return f"EntryPoint({self.name} @ 0x{self.address:08x})"

    def __hash__(self) -> int:
        return self.address

    def __eq__(self, other) -> bool:
        if not isinstance(other, EntryPointInfo):
            return False
        return self.address == other.address


@dataclass
class UdonProgramData:
    byte_code_hex: str
    byte_code_length: int
    symbols: Dict[str, SymbolInfo] = field(default_factory=dict)
    entry_points: List[EntryPointInfo] = field(default_factory=list)
    heap_initial_values: Dict[int, HeapEntry] = field(default_factory=dict)

    _generated_func_id: int = 0
    CLASS_NAME_ADDR: int = 1
    CLASS_NAME_SYMBOL_NAME: str = "__refl_typename"

    def get_symbol_by_address(self, address: int) -> SymbolInfo:
        for symbol in self.symbols.values():
            if symbol.address == address:
                return symbol
        raise Exception(f"Failed to find any symbols at 0x{address:08X}")

    def get_entry_point_by_address(self, address: int) -> EntryPointInfo:
        for entry_point in self.entry_points:
            if entry_point.address == address:
                return entry_point
        raise Exception(f"Failed to find any entry points at 0x{address:08X}")

    def get_initial_heap_value(self, address: int) -> Optional[HeapEntry]:
        return self.heap_initial_values.get(address)

    def get_class_name(self) -> Optional[str]:
        possible_class_name_symbol = self.get_symbol_by_address(
            UdonProgramData.CLASS_NAME_ADDR
        )
        if possible_class_name_symbol is None:
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
        if possible_class_name_entry is None:
            logger.warning(
                "Class name entry not found! The Udon program might be broken!"
            )
            return None
        if not possible_class_name_entry.value.is_serializable:
            logger.warning(
                "Class name entry is not serializable! "
                "The Udon program might be broken!"
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
