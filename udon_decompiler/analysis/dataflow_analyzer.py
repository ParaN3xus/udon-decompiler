from typing import Dict, List, Optional

from udon_decompiler.analysis.cfg import CFGBuilder, ControlFlowGraph
from udon_decompiler.analysis.ir import FunctionIR, IRBuilder
from udon_decompiler.analysis.stack_simulator import (
    StackFrame,
    StackSimulator,
    StackValue,
)
from udon_decompiler.analysis.variable_identifier import Variable, VariableIdentifier
from udon_decompiler.models.instruction import Instruction
from udon_decompiler.models.module_info import UdonModuleInfo
from udon_decompiler.models.program import EntryPointInfo, UdonProgramData
from udon_decompiler.utils.logger import logger


class DataFlowAnalyzer:
    # stack sim, var id, exp construction

    def __init__(
        self,
        program: UdonProgramData,
        instructions: List[Instruction],
    ):
        self.program = program
        self.module_info = UdonModuleInfo()
        self.instructions = instructions

        cfg_builder = CFGBuilder(program, instructions)
        self.cfgs = cfg_builder.build()

        self.function_analyzers: Dict[str, FunctionDataFlowAnalyzer] = {}

    def analyze(self) -> Dict[str, "FunctionDataFlowAnalyzer"]:
        logger.debug("Starting dataflow analysis for all functions...")

        for func_name, cfg in self.cfgs.items():
            logger.debug(f"Analyzing function: {func_name}")

            analyzer = FunctionDataFlowAnalyzer(
                program=self.program,
                cfg=cfg,
            )

            analyzer.analyze()
            self.function_analyzers[func_name] = analyzer

        logger.info(f"Completed dataflow analysis for {len(self.cfgs)} functions")

        return self.function_analyzers

    def get_function_analyzer(
        self, function_name: str
    ) -> Optional["FunctionDataFlowAnalyzer"]:
        return self.function_analyzers.get(function_name)


class FunctionDataFlowAnalyzer:
    def __init__(
        self,
        program: UdonProgramData,
        cfg: ControlFlowGraph,
    ):
        self.program = program
        self.module_info = UdonModuleInfo()
        self.cfg = cfg

        self.stack_simulator: StackSimulator
        self.variable_identifier: VariableIdentifier

        # res
        self.variables: Dict[int, Variable] = {}
        self.ir: Optional[FunctionIR] = None

    def analyze(self) -> None:
        logger.debug(f"Analyzing dataflow for {self.cfg.function_name}...")

        self._simulate_stack()
        self._identify_variables()
        self._build_ir()

        logger.debug(
            f"Dataflow analysis complete for {self.cfg.function_name}: "
            f"{len(self.variables)} variables, "
            f"{0 if self.ir is None else len(self.ir.blocks)} "
            "ir blocks"
        )

    def _simulate_stack(self) -> None:
        logger.debug(f"Simulating stack for {self.cfg.function_name}...")

        self.stack_simulator = StackSimulator(self.program)

        # topological trav
        visited = set()

        def visit_block(block, entry_state: Optional[StackFrame]):
            if block in visited:
                return
            visited.add(block)

            # todo: review this
            halt, exit_state = self.stack_simulator.simulate_block(block, entry_state)

            # recur
            if not halt:
                for successor in self.cfg.get_successors(block):
                    if successor not in visited:
                        visit_block(successor, exit_state.copy())

        entry_point = next(
            (
                ep
                for ep in self.program.entry_points
                if ep.address == self.cfg.entry_block.start_address
            ),
            None,
        )
        if entry_point is None:
            raise Exception("Couldn't find the function's entrypoint!")
        init_state = None
        if entry_point.address == entry_point.call_jump_target:
            init_state = StackFrame(
                [
                    StackValue(
                        value=-1,
                        type_hint=None,
                        source_instruction=None,
                        literal_value=Instruction.HALT_JUMP_ADDR,
                    )
                ]
            )
        visit_block(self.cfg.entry_block, init_state)

        logger.debug(f"Stack simulation complete for {self.cfg.function_name}")

    def _identify_variables(self) -> None:
        logger.debug(f"Identifying variables for {self.cfg.function_name}...")

        self.variable_identifier = VariableIdentifier(
            program=self.program, cfg=self.cfg, stack_simulator=self.stack_simulator
        )

        self.variables = self.variable_identifier.identify()

        logger.debug(
            f"Variable identification complete for {self.cfg.function_name}: "
            f"{len(self.variables)} variables"
        )

    def _build_ir(self) -> None:
        logger.debug(f"Building IR for {self.cfg.function_name}...")

        builder = IRBuilder(
            program=self.program,
            cfg=self.cfg,
            stack_simulator=self.stack_simulator,
            variable_identifier=self.variable_identifier,
        )
        self.ir = builder.build()

        logger.debug(
            f"IR complete for {self.cfg.function_name}: "
            f"{len(self.ir.blocks)} blocks"
        )

    def get_variable(self, address: int) -> Optional[Variable]:
        return self.variables.get(address)

    def get_ir(self) -> FunctionIR:
        if self.ir is None:
            raise Exception(f"IR not built for {self.cfg.function_name}")
        return self.ir
