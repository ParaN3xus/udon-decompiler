from typing import Dict, List, Optional

from udon_decompiler.analysis.cfg import CFGBuilder, ControlFlowGraph
from udon_decompiler.analysis.expression_builder import Expression, ExpressionBuilder
from udon_decompiler.analysis.stack_simulator import StackSimulator
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
        logger.info("Starting dataflow analysis for all functions...")

        for func_name, cfg in self.cfgs.items():
            logger.info(f"Analyzing function: {func_name}")

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
        self.expression_builder: ExpressionBuilder

        # res
        self.variables: Dict[int, Variable] = {}
        # instruction address -> expression
        self.expressions: Dict[int, Expression] = {}

    def analyze(self) -> None:
        logger.info(f"Analyzing dataflow for {self.cfg.function_name}...")

        self._simulate_stack()
        self._identify_variables()
        self._build_expressions()

        logger.info(
            f"Dataflow analysis complete for {self.cfg.function_name}: "
            f"{len(self.variables)} variables, {len(self.expressions)} expressions"
        )

    def _simulate_stack(self) -> None:
        logger.debug(f"Simulating stack for {self.cfg.function_name}...")

        self.stack_simulator = StackSimulator(self.program)

        # topological trav
        visited = set()

        def visit_block(block, entry_state):
            if block in visited:
                return
            visited.add(block)

            # todo: review this
            exit_state = self.stack_simulator.simulate_block(block, entry_state)

            # recur
            for successor in self.cfg.get_successors(block):
                if successor not in visited:
                    visit_block(successor, exit_state.copy())

        visit_block(self.cfg.entry_block, None)

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

    def _build_expressions(self) -> None:
        logger.debug(f"Building expressions for {self.cfg.function_name}...")

        self.expression_builder = ExpressionBuilder(
            program=self.program,
            stack_simulator=self.stack_simulator,
            variable_identifier=self.variable_identifier,
        )

        for block in self.cfg.graph.nodes():
            for instruction in block.instructions:
                expr = self.expression_builder.build_expression_from_instruction(
                    instruction
                )
                if expr:
                    self.expressions[instruction.address] = expr

        logger.debug(
            f"Expression building complete for {self.cfg.function_name}: "
            f"{len(self.expressions)} expressions"
        )

    def get_variable(self, address: int) -> Optional[Variable]:
        return self.variables.get(address)

    def get_expression(self, instruction_address: int) -> Optional[Expression]:
        return self.expressions.get(instruction_address)
