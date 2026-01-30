from __future__ import annotations

from typing import List

from udon_decompiler.codegen.ast_nodes import StatementNode, VariableDeclNode
from udon_decompiler.codegen.scfg_lifters.backedge_lifter import BackedgeLifter
from udon_decompiler.codegen.scfg_lifters.cleanup_lifter import CleanupLifter
from udon_decompiler.codegen.scfg_lifters.control_var_lifter import ControlVarLifter
from udon_decompiler.codegen.scfg_lifters.exit_var_lifter import ExitVarLifter
from udon_decompiler.codegen.scfg_lifters.loop_lifter import LoopLifter
from udon_decompiler.codegen.scfg_lifters.raw_emitter import SCFGRawEmitter
from udon_decompiler.codegen.scfg_lifters.while_true_lifter import WhileTrueLifter


class SCFGLifterPipeline:
    def __init__(self, ast_builder, scfg, name_to_block, switch_branches) -> None:
        self._ast_builder = ast_builder
        self._scfg = scfg
        self._name_to_block = name_to_block
        self._switch_branches = switch_branches

    def emit(self) -> tuple[List[StatementNode], List[VariableDeclNode]]:
        emitter = SCFGRawEmitter(
            self._ast_builder, self._scfg, self._name_to_block, self._switch_branches
        )
        statements, synthetic_decls = emitter.emit()

        statements = LoopLifter().lift(statements)
        statements = BackedgeLifter().lift(statements)
        statements = ExitVarLifter().lift(statements)
        statements = ControlVarLifter().lift(statements)
        statements = CleanupLifter().lift(statements)
        statements = WhileTrueLifter().lift(statements)
        statements = CleanupLifter().lift(statements)

        # drop unused synthetic declarations
        referenced = CleanupLifter.collect_referenced_variable_names(statements)
        synthetic_decls = [
            decl for decl in synthetic_decls if decl.var_name in referenced
        ]

        return statements, synthetic_decls
