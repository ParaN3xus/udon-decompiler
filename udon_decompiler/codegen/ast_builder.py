from typing import List, Optional, Set

from udon_decompiler.analysis.basic_block import BasicBlock, BasicBlockType
from udon_decompiler.analysis.dataflow_analyzer import FunctionDataFlowAnalyzer
from udon_decompiler.analysis.expression_builder import (
    Expression,
    ExpressionType,
    Operator,
)
from udon_decompiler.analysis.scfg_adapter import SCFGAdapter
from udon_decompiler.analysis.variable_identifier import Variable
from udon_decompiler.codegen.ast_nodes import (
    AssignmentNode,
    BlockNode,
    CallNode,
    ConstructionNode,
    ExpressionNode,
    ExpressionStatementNode,
    FunctionNode,
    LiteralNode,
    OperatorNode,
    PropertyAccessNode,
    PropertyAccessType,
    ReturnNode,
    StatementNode,
    TypeNode,
    VariableDeclNode,
    VariableNode,
)
from udon_decompiler.codegen.scfg_emitter import _SCFGEmitter
from udon_decompiler.models.instruction import Instruction, OpCode
from udon_decompiler.models.module_info import ParameterType
from udon_decompiler.models.program import SymbolInfo, UdonProgramData
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
        self._block_labels: dict[BasicBlock, str] = {}
        self._emitted_labels: Set[BasicBlock] = set()
        self._goto_targets: Set[BasicBlock] = set()
        self._block_emit_positions: dict[BasicBlock, tuple[BlockNode, int]] = {}
        self._variables_by_name = {
            var.name: var for var in self.analyzer.variables.values()
        }
        self._internal_call_addresses = {
            addr
            for addr, expr in self.analyzer.expressions.items()
            if expr.expr_type == ExpressionType.INTERNAL_CALL
        }

    def build(self) -> FunctionNode:
        if self.cfg.function_name is None:
            raise Exception(
                "Invalid function! function_name shouldn't be None at this stage!"
            )

        logger.debug(f"Building AST for function {self.cfg.function_name}...")

        func_node: FunctionNode = FunctionNode(
            is_public=self.cfg.is_function_public,
            name=self.cfg.function_name,
        )

        body = BlockNode()

        self._add_variable_declarations(func_node, body)
        scfg_result = SCFGAdapter(self.cfg).build()
        emitter = _SCFGEmitter(
            self,
            scfg_result.scfg,
            scfg_result.name_to_block,
            scfg_result.switch_branches,
        )
        statements = emitter.emit()
        for decl in emitter.synthetic_declarations():
            body.add_statement(decl)
        for stmt in statements:
            body.add_statement(stmt)

        func_node.body = body

        logger.debug(
            f"AST built for {self.cfg.function_name}: {len(body.statements)} statements"
        )

        return func_node

    def _add_variable_declarations(
        self, func_node: FunctionNode, block: BlockNode
    ) -> None:
        from ..analysis.variable_identifier import VariableScope

        for var in self.analyzer.variables.values():
            if var.scope == VariableScope.LOCAL:
                if not self._is_variable_used(var):
                    continue
                decl = VariableDeclNode(
                    var_name=var.name, var_type=var.type_hint or "object"
                )
                block.add_statement(decl)
            elif var.scope == VariableScope.TEMPORARY:
                if not self._is_internal_variable(var):
                    continue
                symbol_name = self._symbol_name_for_variable(var)
                if symbol_name == SymbolInfo.RETURN_JUMP_ADDR_SYMBOL_NAME:
                    continue
                if not self._is_variable_used(var):
                    continue
                if self._is_const_variable(var):
                    continue
                if self._get_inline_expression(var) is not None:
                    continue
                decl = VariableDeclNode(
                    var_name=var.name, var_type=var.type_hint or "object"
                )
                block.add_statement(decl)

    def _extract_condition_from_block(
        self, block: BasicBlock
    ) -> Optional[ExpressionNode]:
        inst = block.last_instruction
        if inst.opcode == OpCode.JUMP_IF_FALSE:
            # get cond var
            state = self.analyzer.stack_simulator.get_instruction_state(inst.address)
            if state and len(state.stack) > 0:
                cond_value = state.peek(0)
                if cond_value:
                    var = self.analyzer.variable_identifier.get_variable(
                        cond_value.value
                    )
                    if var:
                        return self._variable_to_ast(var)

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

    def _translate_block_for_condition(self, block: BasicBlock) -> list[StatementNode]:
        if not block.instructions:
            return []
        instructions = block.instructions
        if block.block_type == BasicBlockType.CONDITIONAL:
            instructions = instructions[:-1]

        result: list[StatementNode] = []
        for inst in instructions:
            stmt = self._translate_instruction(inst)
            if stmt:
                result.append(stmt)
        return result

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
                if self._should_inline_output_expression(expr):
                    return None
                external_call_expr = self._create_external_call_expression(expr)
                return ExpressionStatementNode(
                    expression=external_call_expr, address=inst.address
                )
            case ExpressionType.PROPERTY_ACCESS:
                if self._should_inline_output_expression(expr):
                    return None
                prop_acc_expr = self._create_property_access_expression(expr)
                return ExpressionStatementNode(
                    expression=prop_acc_expr, address=inst.address
                )
            case ExpressionType.CONSTRUCTOR:
                if self._should_inline_output_expression(expr):
                    return None
                ctor_expr = self._create_construction_expression(expr)
                return ExpressionStatementNode(
                    expression=ctor_expr, address=inst.address
                )
            case ExpressionType.OPERATOR:
                if self._should_inline_output_expression(expr):
                    return None
                op_expr = self._create_operator_expression(expr)
                return ExpressionStatementNode(expression=op_expr, address=inst.address)
            case ExpressionType.LITERAL | ExpressionType.VARIABLE:
                raise Exception("Unexpected orphan literl or variable expression!")

        # todo: other expr type ignored
        return None

    def _create_assignment_statement(
        self, expr: Expression, inst: Instruction
    ) -> Optional[AssignmentNode]:
        target = expr.value if expr.value else "<unknown>"
        target_var = self._get_variable_by_name(target)
        if target_var and not self._should_emit_assignment(target_var):
            return None

        value_expr = None
        if expr.arguments:
            value_expr = self._expression_to_ast(expr.arguments[0])

        return AssignmentNode(target=target, value=value_expr, address=inst.address)

    def _create_internal_call_expression(self, expr: Expression) -> CallNode:
        if expr.entry_point is None:
            raise Exception("Invalid internal call expression! entry_point expected!")
        if expr.entry_point.name is None:
            raise Exception(
                "Invalid internal call expression! "
                "function_name shouldn't be null at this stage!"
            )

        return CallNode(
            is_external=False,
            function_name=expr.entry_point.name,
            arguments=[],
        )

    def _create_external_call_expression(
        self,
        expr: Expression,
        as_value: bool = False,
        visited: Optional[Set[str]] = None,
    ) -> CallNode:
        if expr.function_info is None:
            raise Exception("Invalid external call expression! function_info expected!")

        if expr.function_info.original_name is None:
            raise Exception("Invalid function_info! original_name expected!")

        if visited is None:
            visited = set()

        args: List[tuple[ParameterType, ExpressionNode]] = []
        receiver = None
        raw_args = list(expr.arguments)

        if not expr.function_info.returns_void and raw_args:
            receiver_expr = raw_args.pop()
            if not as_value and receiver_expr.expr_type == ExpressionType.VARIABLE:
                receiver_var = self._get_variable_by_name(str(receiver_expr.value))
                if not receiver_var:
                    raise Exception("receiver_var expected!")
                receiver = self._variable_to_receiver_ast(receiver_var)

        for i, arg in enumerate(raw_args):
            args.append(
                (
                    expr.function_info.parameters[i],
                    self._expression_to_ast(arg, visited, as_value=True),
                )
            )

        return CallNode(
            is_external=True,
            type_name=expr.function_info.type_name,
            function_name=expr.function_info.function_name,
            original_name=expr.function_info.original_name,
            is_static=expr.function_info.is_static,
            returns_void=expr.function_info.returns_void,
            receiver=receiver,
            emit_as_expression=as_value,
            arguments=args,
        )

    def _create_operator_expression(
        self,
        expr: Expression,
        as_value: bool = False,
        visited: Optional[Set[str]] = None,
    ) -> OperatorNode:
        if expr.function_info is None:
            raise Exception("Invalid call expression! function_info expected!")

        if expr.operator is None:
            raise Exception("Invalid operator expression! A valid operator expected!")

        if visited is None:
            visited = set()

        raw_args = list(expr.arguments)
        receiver = None
        if raw_args:
            receiver_expr = raw_args.pop()
            if not as_value and receiver_expr.expr_type == ExpressionType.VARIABLE:
                receiver_var = self._get_variable_by_name(str(receiver_expr.value))
                if not receiver_var:
                    raise Exception("receiver_var expected!")
                receiver = self._variable_to_receiver_ast(receiver_var)

        oprs = [
            self._expression_to_ast(arg, visited, as_value=True) for arg in raw_args
        ]

        if expr.operator == Operator.Conversion:
            oprs.insert(0, TypeNode(type_name=expr.function_info.type_name))

        return OperatorNode(
            operator=expr.operator,
            operands=oprs,
            receiver=receiver,
            emit_as_expression=as_value,
        )

    def _create_property_access_expression(
        self,
        expr: Expression,
        as_value: bool = False,
        visited: Optional[Set[str]] = None,
    ) -> PropertyAccessNode:
        if expr.function_info is None:
            raise Exception("Invalid call expression! function_info expected!")
        if expr.function_info.original_name is None:
            raise Exception("Invalid function_info! original_name expected!")

        try:
            type = PropertyAccessType(
                expr.function_info.function_name[: PropertyAccessType.literal_len()]
            )
        except ValueError as e:
            logger.error(
                f"Failed to construct PropertyAccessType from "
                f"'{expr.function_info.function_name}'"
            )
            raise e

        if visited is None:
            visited = set()

        # get: (this), receiver
        # set: (this), provider
        raw_args = list(expr.arguments)

        if len(raw_args) > 2 or len(raw_args) < 1:
            raise Exception(
                "Invalid property access expression! Unexpected arguments count!"
            )

        is_static = len(raw_args) == 1

        this_expr = None
        output_arg = None
        if is_static:
            output_arg = raw_args[0]
        else:
            output_arg = raw_args[1]

            this_expr = (
                TypeNode(type_name=expr.function_info.type_name)
                if is_static
                else self._expression_to_ast(raw_args[0], visited, as_value=True)
            )

        target = None
        value = None
        if type == PropertyAccessType.GET:
            if output_arg is not None:
                if output_arg.expr_type == ExpressionType.VARIABLE:
                    receiver_var = self._get_variable_by_name(str(output_arg.value))
                    if not receiver_var:
                        raise Exception("receiver_var expected!")
                    target = self._variable_to_receiver_ast(receiver_var)
                else:
                    target = self._expression_to_ast(output_arg, visited, as_value=True)
        elif type == PropertyAccessType.SET:
            if output_arg is not None:
                value = self._expression_to_ast(output_arg, visited, as_value=True)

        return PropertyAccessNode(
            access_type=type,
            is_static=is_static,
            field=expr.function_info.original_name,
            type_name=expr.function_info.type_name,
            this=this_expr,
            target=target,
            value=value,
            emit_as_expression=as_value,
        )

    def _create_construction_expression(
        self,
        expr: Expression,
        as_value: bool = False,
        visited: Optional[Set[str]] = None,
    ) -> ConstructionNode:
        if expr.function_info is None:
            raise Exception("Invalid call expression! function_info expected!")

        if visited is None:
            visited = set()

        raw_args = list(expr.arguments)
        receiver = None
        if not raw_args:
            raise Exception(
                "Invalid construction expression! At least one argument expected!"
            )
        receiver_expr = raw_args.pop()
        if not as_value and receiver_expr.expr_type == ExpressionType.VARIABLE:
            receiver_var = self._get_variable_by_name(str(receiver_expr.value))
            if not receiver_var:
                raise Exception("receiver_var expected!")
            receiver = self._variable_to_receiver_ast(receiver_var)

        args = [
            self._expression_to_ast(arg, visited, as_value=True) for arg in raw_args
        ]

        return ConstructionNode(
            type_name=expr.function_info.type_name,
            arguments=args,
            receiver=receiver,
            emit_as_expression=as_value,
        )

    def _expression_to_ast(
        self,
        expr: Expression,
        visited: Optional[Set[str]] = None,
        as_value: bool = False,
    ) -> ExpressionNode:
        if visited is None:
            visited = set()
        if expr.expr_type == ExpressionType.LITERAL:
            return LiteralNode(value=expr.value, literal_type=expr.type_hint)
        elif expr.expr_type == ExpressionType.VARIABLE:
            var = self._get_variable_by_name(str(expr.value))
            if not var:
                raise Exception("Invalid variable expression!")
            return self._variable_to_ast(var, visited)
        elif expr.expr_type == ExpressionType.EXTERNAL_CALL:
            return self._create_external_call_expression(
                expr, as_value=as_value, visited=visited
            )
        elif expr.expr_type == ExpressionType.OPERATOR:
            return self._create_operator_expression(
                expr, as_value=as_value, visited=visited
            )
        elif expr.expr_type == ExpressionType.PROPERTY_ACCESS:
            return self._create_property_access_expression(
                expr, as_value=as_value, visited=visited
            )
        elif expr.expr_type == ExpressionType.CONSTRUCTOR:
            return self._create_construction_expression(
                expr, as_value=as_value, visited=visited
            )
        else:
            return LiteralNode(value=f"<{expr.expr_type.value}>")

    def _get_variable_by_name(self, name: str) -> Variable | None:
        return self._variables_by_name.get(name)

    def _variable_to_ast(
        self, var: Variable, visited: Optional[Set[str]] = None
    ) -> ExpressionNode:
        if visited is None:
            visited = set()
        if var.name in visited:
            return VariableNode(var_name=var.name, var_type=var.type_hint)

        visited.add(var.name)

        # const -> literal
        if self._is_const_variable(var):
            literal = self._literal_from_variable(var)
            if literal:
                visited.remove(var.name)
                return literal

        # internal (temp) -> expr
        if self._is_internal_variable(var):
            # the expr that writes into the var
            inline_expr = self._get_inline_expression(var)
            if inline_expr:
                node = self._expression_to_ast(inline_expr, visited, as_value=True)
                visited.remove(var.name)
                return node

        visited.remove(var.name)
        return VariableNode(var_name=var.name, var_type=var.type_hint)

    def _symbol_name_for_variable(self, var) -> str:
        if var.original_symbol:
            return var.original_symbol.name
        return var.name

    def _variable_to_plain_ast(self, var: Variable) -> VariableNode:
        return VariableNode(var_name=var.name, var_type=var.type_hint)

    def _variable_to_receiver_ast(self, var: Variable) -> ExpressionNode:
        if self._is_const_variable(var):
            return self._variable_to_ast(var)
        return self._variable_to_plain_ast(var)

    def _is_variable_used(self, var: Variable) -> bool:
        return bool(var.read_locations or var.write_locations)

    def _is_const_variable(self, var: Variable) -> bool:
        symbol_name = self._symbol_name_for_variable(var)
        return symbol_name.startswith(SymbolInfo.CONST_SYMBOL_PREFIX)

    def _is_internal_variable(self, var: Variable) -> bool:
        symbol_name = self._symbol_name_for_variable(var)
        return symbol_name.startswith(
            SymbolInfo.INTERNAL_SYMBOL_PREFIX
        ) or symbol_name.startswith(SymbolInfo.GLOBAL_INTERNAL_SYMBOL_PREFIX)

    def _should_emit_assignment(self, var: Variable) -> bool:
        if self._is_const_variable(var):
            return False

        if self._is_internal_variable(var):
            if len(var.read_locations) == 0:
                return False
            if self._get_inline_expression(var):
                return False

        return True

    def _literal_from_variable(self, var) -> Optional[LiteralNode]:
        heap_entry = self.program.get_initial_heap_value(var.address)
        if heap_entry is None or heap_entry.value is None:
            return None
        if not heap_entry.value.is_serializable:
            return None

        value = heap_entry.value.value
        if isinstance(value, (list, dict)):
            return None

        return LiteralNode(value=value, literal_type=var.type_hint)

    def _get_inline_expression(self, var: Variable) -> Optional[Expression]:
        # 1w nr, or it's not a inline expr
        if len(var.write_locations) != 1 or len(var.read_locations) == 0:
            return None

        write_addr = next(iter(var.write_locations))
        expr = self.analyzer.get_expression(write_addr)
        if expr is None:
            raise Exception("Invalid write_locations. Expressions expected!")
        if expr.expr_type == ExpressionType.ASSIGNMENT:
            if expr.value != var.name:
                return None
            if not expr.arguments:
                return None
            # Avoid inlining temp = src when src may change before temp is read.
            arg = expr.arguments[0]
            if arg.expr_type == ExpressionType.VARIABLE:
                source_var = self._get_variable_by_name(str(arg.value))
                if source_var:
                    for read_addr in var.read_locations:
                        if read_addr <= write_addr:
                            continue
                        if any(
                            write_addr < w < read_addr
                            for w in source_var.write_locations
                        ):
                            return None
                        if any(
                            write_addr < call_addr < read_addr
                            for call_addr in self._internal_call_addresses
                        ):
                            return None
            return expr.arguments[0]

        if expr.expr_type in (
            ExpressionType.EXTERNAL_CALL,
            ExpressionType.OPERATOR,
            ExpressionType.PROPERTY_ACCESS,
            ExpressionType.CONSTRUCTOR,
        ):
            if not expr.arguments:
                return None
            # the receiver
            last_arg = expr.arguments[-1]
            if (
                last_arg.expr_type == ExpressionType.VARIABLE
                and last_arg.value == var.name
            ):
                return expr
            return None

        return None

    def _should_inline_output_expression(self, expr: Expression) -> bool:
        # if there's no receiver(the last argument), it can't be inlined
        if not expr.arguments:
            return False

        # receiver must be a variable
        last_arg = expr.arguments[-1]
        if last_arg.expr_type != ExpressionType.VARIABLE:
            return False

        # receiver must be internal(temp)
        var = self._get_variable_by_name(str(last_arg.value))
        if var is None or not self._is_internal_variable(var):
            return False

        # receiver is indeed a inline expr
        inline_expr = self._get_inline_expression(var)
        return inline_expr is expr
