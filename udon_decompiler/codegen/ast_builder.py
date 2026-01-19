from typing import List, Optional, Set

from udon_decompiler.analysis.basic_block import BasicBlock
from udon_decompiler.analysis.control_flow import ControlStructure, ControlStructureType
from udon_decompiler.analysis.dataflow_analyzer import FunctionDataFlowAnalyzer
from udon_decompiler.analysis.expression_builder import Expression, ExpressionType
from udon_decompiler.codegen.ast_nodes import (
    AssignmentNode,
    BlockNode,
    CallNode,
    DoWhileNode,
    ExpressionNode,
    ExpressionStatementNode,
    FunctionNode,
    IfElseNode,
    IfNode,
    LiteralNode,
    StatementNode,
    VariableDeclNode,
    VariableNode,
    WhileNode,
)
from udon_decompiler.models.instruction import Instruction
from udon_decompiler.models.program import UdonProgramData
from udon_decompiler.utils.logger import logger


class ASTBuilder:
    def __init__(
        self, program: UdonProgramData, function_analyzer: FunctionDataFlowAnalyzer
    ):
        self.program = program
        self.analyzer = function_analyzer
        self.cfg = function_analyzer.cfg

        self._processed_blocks: Set[BasicBlock] = set()

        self._label_counter = 0

    def build(self) -> FunctionNode:
        logger.info(f"Building AST for function {self.cfg.function_name}...")

        func_node = FunctionNode(
            name=self.cfg.function_name or "unknown_function",
            return_type="void",
        )

        from ..analysis.control_flow import ControlFlowStructureIdentifier

        struct_identifier = ControlFlowStructureIdentifier(self.cfg)
        structures = struct_identifier.identify()

        body = BlockNode()

        self._add_variable_declarations(body)

        self._build_block_statements(self.cfg.entry_block, body, structures)

        func_node.body = body

        logger.info(
            f"AST built for {self.cfg.function_name}: {len(body.statements)} statements"
        )

        return func_node

    def _add_variable_declarations(self, block: BlockNode) -> None:
        from ..analysis.variable_identifier import VariableScope

        for var in self.analyzer.variables.values():
            if var.scope in (
                VariableScope.LOCAL,
                VariableScope.TEMPORARY,
                VariableScope.PARAMETER,
            ):
                decl = VariableDeclNode(
                    var_name=var.name, var_type=var.type_hint or "object"
                )
                block.add_statement(decl)

    def _build_block_statements(
        self,
        block: BasicBlock,
        parent_block: BlockNode,
        structures: List[ControlStructure],
        visited: Optional[Set[BasicBlock]] = None,
    ) -> None:
        if visited is None:
            visited = set()

        if block in visited:
            return

        visited.add(block)

        structure = self._find_structure_with_header(block, structures)

        if structure:
            self._build_control_structure(structure, parent_block, structures, visited)
        else:
            self._translate_basic_block(block, parent_block)

            successors = list(self.cfg.get_successors(block))

            if len(successors) == 1:
                self._build_block_statements(
                    successors[0], parent_block, structures, visited
                )
            elif len(successors) > 1:
                # shouldn't happen
                logger.warning(
                    f"Block 0x{block.start_address:08x} has multiple successors but no control structure"
                )

    def _find_structure_with_header(
        self, block: BasicBlock, structures: List[ControlStructure]
    ) -> Optional[ControlStructure]:
        for structure in structures:
            if structure.header == block:
                return structure
        return None

    def _build_control_structure(
        self,
        structure: ControlStructure,
        parent_block: BlockNode,
        all_structures: List[ControlStructure],
        visited: Set[BasicBlock],
    ) -> None:
        if structure.type == ControlStructureType.IF:
            self._build_if_statement(structure, parent_block, all_structures, visited)

        elif structure.type == ControlStructureType.IF_ELSE:
            self._build_if_else_statement(
                structure, parent_block, all_structures, visited
            )

        elif structure.type == ControlStructureType.WHILE:
            self._build_while_statement(
                structure, parent_block, all_structures, visited
            )

        elif structure.type == ControlStructureType.DO_WHILE:
            self._build_do_while_statement(
                structure, parent_block, all_structures, visited
            )

        if structure.exit and structure.exit not in visited:
            self._build_block_statements(
                structure.exit, parent_block, all_structures, visited
            )

    def _build_if_statement(
        self,
        structure: ControlStructure,
        parent_block: BlockNode,
        all_structures: List[ControlStructure],
        visited: Set[BasicBlock],
    ) -> None:
        condition = self._extract_condition_from_block(structure.header)

        then_block = BlockNode()

        for block in structure.true_branch:
            if block not in visited:
                self._build_block_statements(block, then_block, all_structures, visited)

        if_node = IfNode(
            condition=condition,
            then_block=then_block,
            address=structure.header.start_address,
        )

        parent_block.add_statement(if_node)

    def _build_if_else_statement(
        self,
        structure: ControlStructure,
        parent_block: BlockNode,
        all_structures: List[ControlStructure],
        visited: Set[BasicBlock],
    ) -> None:
        condition = self._extract_condition_from_block(structure.header)

        then_block = BlockNode()
        else_block = BlockNode()

        for block in structure.true_branch:
            if block not in visited:
                self._build_block_statements(block, then_block, all_structures, visited)

        if structure.false_branch:
            for block in structure.false_branch:
                if block not in visited:
                    self._build_block_statements(
                        block, else_block, all_structures, visited
                    )

        if_else_node = IfElseNode(
            condition=condition,
            then_block=then_block,
            else_block=else_block,
            address=structure.header.start_address,
        )

        parent_block.add_statement(if_else_node)

    def _build_while_statement(
        self,
        structure: ControlStructure,
        parent_block: BlockNode,
        all_structures: List[ControlStructure],
        visited: Set[BasicBlock],
    ) -> None:
        condition = self._extract_condition_from_block(structure.header)

        body = BlockNode()

        if structure.loop_body:
            for block in structure.loop_body:
                if block not in visited:
                    self._build_block_statements(block, body, all_structures, visited)

        while_node = WhileNode(
            condition=condition, body=body, address=structure.header.start_address
        )

        parent_block.add_statement(while_node)

    def _build_do_while_statement(
        self,
        structure: ControlStructure,
        parent_block: BlockNode,
        all_structures: List[ControlStructure],
        visited: Set[BasicBlock],
    ) -> None:
        condition = self._extract_condition_from_block(structure.header)

        body = BlockNode()

        if structure.loop_body:
            for block in structure.loop_body:
                if block not in visited:
                    self._build_block_statements(block, body, all_structures, visited)

        do_while_node = DoWhileNode(
            condition=condition, body=body, address=structure.header.start_address
        )

        parent_block.add_statement(do_while_node)

    def _extract_condition_from_block(
        self, block: BasicBlock
    ) -> Optional[ExpressionNode]:
        for inst in block.instructions:
            if inst.opcode.name == "JUMP_IF_FALSE":
                # get cond var
                state = self.analyzer.stack_simulator.get_instruction_state(
                    inst.address
                )
                if state and len(state.stack) > 0:
                    cond_value = state.peek(0)
                    if cond_value:
                        var = self.analyzer.variable_identifier.get_variable(
                            cond_value.value
                        )
                        if var:
                            return VariableNode(
                                var_name=var.name, var_type=var.type_hint
                            )

        return VariableNode(var_name="<condition>")

    def _translate_basic_block(
        self, block: BasicBlock, parent_block: BlockNode
    ) -> None:
        for inst in block.instructions:
            stmt = self._translate_instruction(inst)
            if stmt:
                parent_block.add_statement(stmt)

    def _translate_instruction(self, inst: Instruction) -> Optional[StatementNode]:
        expr = self.analyzer.get_expression(inst.address)

        if not expr:
            return None

        if expr.expr_type == ExpressionType.ASSIGNMENT:
            return self._create_assignment_statement(expr, inst)

        elif expr.expr_type == ExpressionType.CALL:
            call_expr = self._create_call_expression(expr)
            return ExpressionStatementNode(expression=call_expr, address=inst.address)

        # todo: other expr type ignored
        return None

    def _create_assignment_statement(
        self, expr: Expression, inst: Instruction
    ) -> AssignmentNode:
        target = expr.value if expr.value else "<unknown>"

        value_expr = None
        if expr.operands:
            value_expr = self._convert_expression_to_ast(expr.operands[0])

        return AssignmentNode(target=target, value=value_expr, address=inst.address)

    def _create_call_expression(self, expr: Expression) -> CallNode:
        args = [self._convert_expression_to_ast(arg) for arg in expr.arguments]

        return CallNode(function_info=expr.function_info, arguments=args)

    def _convert_expression_to_ast(self, expr: Expression) -> ExpressionNode:
        if expr.expr_type == ExpressionType.LITERAL:
            return LiteralNode(value=expr.value, literal_type=expr.type_hint)

        elif expr.expr_type == ExpressionType.VARIABLE:
            return VariableNode(var_name=str(expr.value), var_type=expr.type_hint)

        elif expr.expr_type == ExpressionType.CALL:
            return self._create_call_expression(expr)

        else:
            return LiteralNode(value=f"<{expr.expr_type.value}>")

    def _generate_label(self) -> str:
        label = f"label_{self._label_counter}"
        self._label_counter += 1
        return label
