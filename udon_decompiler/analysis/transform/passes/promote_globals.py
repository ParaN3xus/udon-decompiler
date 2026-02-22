from __future__ import annotations

from typing import Dict, List

from udon_decompiler.analysis.ir.nodes import IRFunction, IRVariableDeclarationStatement
from udon_decompiler.analysis.transform.pass_base import (
    IProgramTransform,
    ProgramTransformContext,
)
from udon_decompiler.analysis.variable_identifier import VariableScope


class PromoteGlobals(IProgramTransform):
    """Promote globals and shared locals into class-level declarations."""

    def run(
        self,
        functions: List[IRFunction],
        context: ProgramTransformContext,
    ) -> None:
        if context.ir_class is None:
            return

        address_to_functions: Dict[int, set[int]] = {}
        address_to_decl: Dict[int, IRVariableDeclarationStatement] = {}
        forced_global_addresses: set[int] = set()

        for fn_index, function in enumerate(functions):
            for declaration in function.variable_declarations:
                addr = declaration.variable.address
                address_to_functions.setdefault(addr, set()).add(fn_index)
                if declaration.variable.scope == VariableScope.GLOBAL:
                    forced_global_addresses.add(addr)
                existing = address_to_decl.get(addr)
                if existing is None or (
                    existing.init_value is None and declaration.init_value is not None
                ):
                    address_to_decl[addr] = declaration

        promoted_addresses = {
            addr for addr, owners in address_to_functions.items() if len(owners) >= 2
        }
        promoted_addresses.update(forced_global_addresses)

        for function in functions:
            function.variable_declarations = [
                decl
                for decl in function.variable_declarations
                if decl.variable.address not in promoted_addresses
            ]

        context.ir_class.variable_declarations = [
            address_to_decl[addr] for addr in sorted(promoted_addresses)
        ]
