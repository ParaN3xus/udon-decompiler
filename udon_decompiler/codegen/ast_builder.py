from typing import List, Optional, Set

from udon_decompiler.analysis.basic_block import BasicBlock, BasicBlockType
from udon_decompiler.analysis.control_flow import (
    ControlFlowStructureIdentifier,
    ControlStructure,
    ControlStructureType,
)
from udon_decompiler.analysis.dataflow_analyzer import FunctionDataFlowAnalyzer
from udon_decompiler.analysis.expression_builder import Expression, ExpressionType
from udon_decompiler.codegen.ast_nodes import (
    AssignmentNode,
    BlockNode,
    CallNode,
    ConstructionNode,
    DoWhileNode,
    ExpressionNode,
    ExpressionStatementNode,
    FunctionNode,
    IfElseNode,
    IfNode,
    LiteralNode,
    OperatorNode,
    PropertyAccessNode,
    PropertyAccessType,
    ReturnNode,
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
        if self.cfg.function_name is None:
            raise Exception(
                "Invalid function! function_name shouldn't be None at this stage!"
            )

        logger.info(f"Building AST for function {self.cfg.function_name}...")

        func_node: FunctionNode = FunctionNode(
            name=self.cfg.function_name,
            return_type="void",
        )

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
                    f"Block 0x{block.start_address:08x} has multiple successors but no "
                    + "control structure"
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

        if block.block_type == BasicBlockType.RETURN:
            parent_block.add_statement(ReturnNode())

    def _translate_instruction(self, inst: Instruction) -> Optional[StatementNode]:
        expr = self.analyzer.get_expression(inst.address)

        if expr is None:
            return None

        match expr.expr_type:
            case ExpressionType.ASSIGNMENT:
                return self._create_assignment_statement(expr, inst)
            case ExpressionType.INTERNAL_CALL:
                internal_call_expr = self._create_internal_call_expression(expr)
                return ExpressionStatementNode(
                    expression=internal_call_expr, address=inst.address
                )
            case ExpressionType.EXTERNAL_CALL:
                external_call_expr = self._create_external_call_expression(expr)
                return ExpressionStatementNode(
                    expression=external_call_expr, address=inst.address
                )
            case ExpressionType.PROPERTY_ACCESS:
                prop_acc_expr = self._create_property_access_expression(expr)
                return ExpressionStatementNode(
                    expression=prop_acc_expr, address=inst.address
                )
            case ExpressionType.CONSTRUCTOR:
                ctor_expr = self._create_construction_expression(expr)
                return ExpressionStatementNode(
                    expression=ctor_expr, address=inst.address
                )
            case ExpressionType.OPERATOR:
                op_expr = self._create_operator_expression(expr)
                return ExpressionStatementNode(expression=op_expr, address=inst.address)
            case ExpressionType.LITERAL | ExpressionType.VARIABLE:
                raise Exception("Unexpected orphan literl or variable expression!")

        # todo: other expr type ignored
        return None

    def _create_assignment_statement(
        self, expr: Expression, inst: Instruction
    ) -> AssignmentNode:
        target = expr.value if expr.value else "<unknown>"

        value_expr = None
        if expr.arguments:
            value_expr = self._convert_expression_to_ast(expr.arguments[0])

        return AssignmentNode(target=target, value=value_expr, address=inst.address)

    def _create_internal_call_expression(self, expr: Expression) -> CallNode:
        if expr.entry_point is None:
            raise Exception("Invalid internal call expression! entry_point expected!")
        if expr.entry_point.name is None:
            raise Exception(
                "Invalid internal call expression! "
                + "function_name shouldn't be null at this stage!"
            )

        return CallNode(
            is_external=False,
            function_name=expr.entry_point.name,
            arguments=[],
        )

    def _create_external_call_expression(self, expr: Expression) -> CallNode:
        if expr.function_info is None:
            raise Exception("Invalid external call expression! function_info expected!")

        args = [self._convert_expression_to_ast(arg) for arg in expr.arguments]

        is_static = (
            expr.function_info.is_static
            if expr.function_info.is_static is not None
            else True
        )

        returns_void = (
            expr.function_info.returns_void
            if expr.function_info.returns_void is not None
            else True
        )

        return CallNode(
            is_external=True,
            type_name=expr.function_info.type_name,
            function_name=expr.function_info.function_name,
            original_name=expr.function_info.original_name,
            is_static=is_static,
            returns_void=returns_void,
            arguments=args,
        )

    def _create_operator_expression(self, expr: Expression) -> OperatorNode:
        if expr.function_info is None:
            raise Exception("Invalid call expression! function_info expected!")

        if expr.operator is None:
            raise Exception("Invalid operator expression! A valid operator expected!")

        oprs = [self._convert_expression_to_ast(arg) for arg in expr.arguments]
        receiver = oprs.pop()

        return OperatorNode(
            operator=expr.operator,
            operands=oprs,
            receiver=receiver,
        )

    def _create_property_access_expression(
        self, expr: Expression
    ) -> PropertyAccessNode:
        if expr.function_info is None:
            raise Exception("Invalid call expression! function_info expected!")

        args = [self._convert_expression_to_ast(arg) for arg in expr.arguments]

        try:
            type = PropertyAccessType(
                expr.function_info.function_name[: PropertyAccessType.literal_len()]
            )
        except ValueError as e:
            logger.error(
                f"Failed to construct PropertyAccessType from "
                + f"'{expr.function_info.function_name}'"
            )
            raise e

        return PropertyAccessNode(
            type=type,
            is_static=len(args) == 1,
            field=expr.function_info.original_name,
            type_name=expr.function_info.type_name,
            receiver=args[-1],
            this=args[0],
        )

    def _create_construction_expression(self, expr: Expression) -> ConstructionNode:
        if expr.function_info is None:
            raise Exception("Invalid call expression! function_info expected!")

        args = [self._convert_expression_to_ast(arg) for arg in expr.arguments]
        receiver = args.pop()

        return ConstructionNode(
            type_name=expr.function_info.type_name, arguments=args, receiver=receiver
        )

    def _convert_expression_to_ast(self, expr: Expression) -> ExpressionNode:
        if expr.expr_type == ExpressionType.LITERAL:
            return LiteralNode(value=expr.value, literal_type=expr.type_hint)
        elif expr.expr_type == ExpressionType.VARIABLE:
            return VariableNode(var_name=str(expr.value), var_type=expr.type_hint)
        elif expr.expr_type == ExpressionType.EXTERNAL_CALL:
            return self._create_external_call_expression(expr)
        elif expr.expr_type == ExpressionType.OPERATOR:
            return self._create_operator_expression(expr)
        elif expr.expr_type == ExpressionType.PROPERTY_ACCESS:
            return self._create_property_access_expression(expr)
        elif expr.expr_type == ExpressionType.CONSTRUCTOR:
            return self._create_construction_expression(expr)
        else:
            return LiteralNode(value=f"<{expr.expr_type.value}>")

    def _generate_label(self) -> str:
        label = f"label_{self._label_counter}"
        self._label_counter += 1
        return label
