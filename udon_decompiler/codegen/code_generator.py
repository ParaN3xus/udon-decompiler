import json
import shutil
import subprocess
from typing import Optional

from udon_decompiler.analysis.dataflow_analyzer import FunctionDataFlowAnalyzer
from udon_decompiler.analysis.expression_builder import Operator
from udon_decompiler.analysis.variable_identifier import Variable
from udon_decompiler.codegen.ast_nodes import (
    AssignmentNode,
    BlockNode,
    BreakNode,
    CallNode,
    ConstructionNode,
    ContinueNode,
    DoWhileNode,
    ExpressionNode,
    ExpressionStatementNode,
    FunctionNode,
    GotoNode,
    IfElseNode,
    IfNode,
    LabelNode,
    LiteralNode,
    OperatorNode,
    ProgramNode,
    PropertyAccessNode,
    PropertyAccessType,
    ReturnNode,
    StatementNode,
    SwitchCaseNode,
    SwitchNode,
    TypeNode,
    VariableDeclNode,
    VariableNode,
    WhileNode,
)
from udon_decompiler.models.module_info import ParameterType
from udon_decompiler.models.program import HeapEntry, SymbolInfo, UdonProgramData
from udon_decompiler.utils.logger import logger


class CSharpCodeGenerator:
    _UNARY_OPERATORS = {
        Operator.UnaryMinus,
        Operator.UnaryNegation,
        Operator.Conversion,
    }
    _ASSOCIATIVE_OPERATORS = {
        Operator.Addition,
        Operator.Multiplication,
        Operator.LogicalAnd,
        Operator.LogicalOr,
        Operator.LogicalXor,
    }

    def _block_has_statements(self, block: Optional[BlockNode]) -> bool:
        return bool(block and block.statements)

    def _is_single_if_statement_block(self, block: BlockNode) -> bool:
        if len(block.statements) != 1:
            return False
        return isinstance(block.statements[0], (IfNode, IfElseNode))

    def _invert_condition(self, condition: Optional[ExpressionNode]) -> ExpressionNode:
        if condition is None:
            return LiteralNode(literal_type="System.Boolean", value=True)
        if (
            isinstance(condition, OperatorNode)
            and condition.operator == Operator.UnaryNegation
            and condition.operands
        ):
            return condition.operands[0]
        return OperatorNode(
            operator=Operator.UnaryNegation,
            receiver=None,
            operands=[condition],
        )

    def _operator_precedence(self, op: Operator) -> int:
        # Higher number = higher precedence.
        if op in self._UNARY_OPERATORS:
            return 7
        if op in (Operator.Multiplication, Operator.Division, Operator.Remainder):
            return 6
        if op in (Operator.Addition, Operator.Subtraction):
            return 5
        if op in (Operator.LeftShift, Operator.RightShift):
            return 4
        if op in (
            Operator.GreaterThan,
            Operator.GreaterThanOrEqual,
            Operator.LessThan,
            Operator.LessThanOrEqual,
        ):
            return 3
        if op in (Operator.Equality, Operator.Inequality):
            return 2
        if op == Operator.LogicalAnd:
            return 1
        if op == Operator.LogicalXor:
            return 0
        if op == Operator.LogicalOr:
            return -1
        return -2

    def _operator_needs_parentheses(
        self,
        parent_op: Operator,
        child_expr: ExpressionNode,
        operand_index: int,
    ) -> bool:
        if not isinstance(child_expr, OperatorNode):
            return False

        parent_prec = self._operator_precedence(parent_op)
        child_prec = self._operator_precedence(child_expr.operator)

        if child_prec < parent_prec:
            return True
        if child_prec > parent_prec:
            return False

        # Same precedence: be conservative on the right-hand side.
        if parent_op in self._UNARY_OPERATORS:
            # Avoid "--a" and "!-a" ambiguities.
            return True

        if operand_index == 1:
            if child_expr.operator != parent_op:
                return True
            return parent_op not in self._ASSOCIATIVE_OPERATORS

        return False

    def _generate_operator_operand(
        self, parent_op: Operator, operand: ExpressionNode, operand_index: int
    ) -> str:
        operand_str = self._generate_expression(operand)
        if self._operator_needs_parentheses(parent_op, operand, operand_index):
            return f"({operand_str})"
        return operand_str

    def generate(self, program_node: ProgramNode, class_name: str) -> str:
        logger.info(f"Generating C# code for {class_name}...")

        lines = [
            "// Decompiled Udon Program",
            "// This is pseudo-code and may not compile directly",
            "",
        ]

        class_lines = [
            f"public class {class_name} : UdonSharpBehaviour",
            "{",
        ]

        for global_var in program_node.global_variables:
            decl_lines = self._generate_variable_decl(global_var)
            class_lines.extend(decl_lines)

        if program_node.global_variables:
            class_lines.append("")

        for func_node in program_node.functions:
            func_lines = self._generate_function(func_node)
            class_lines.extend(func_lines)
            class_lines.append("")

        if program_node.functions:
            class_lines.pop()

        class_lines.append("}")

        if program_node.namespace:
            lines.extend(
                [
                    f"namespace {program_node.namespace}",
                    "{",
                ]
            )
            lines.extend(class_lines)
            lines.append("}")
        else:
            lines.extend(class_lines)

        code = "\n".join(lines)

        return self._format(code)

    def _format(self, code: str) -> str:
        clang_format = shutil.which("clang-format")
        if clang_format is None:
            raise RuntimeError(
                "clang-format not found on PATH. Install it or adjust PATH."
            )

        style = (
            "{BasedOnStyle: Google, Language: CSharp, "
            "IndentWidth: 4, ColumnLimit: 160, "
            "BreakBeforeBraces: Allman}"
        )
        result = subprocess.run(
            [clang_format, "--assume-filename=_.cs", f"-style={style}"],
            input=code,
            text=True,
            capture_output=True,
            check=False,
            encoding="utf-8",
        )
        if result.returncode != 0:
            stderr = result.stderr.strip() or "unknown error"
            raise RuntimeError(f"clang-format failed: {stderr}")
        return result.stdout

    def _generate_function(self, func_node: FunctionNode) -> list[str]:
        logger.debug(f"Generating C# code for {func_node.name}...")

        lines = []

        signature = self._generate_function_signature(func_node)
        lines.append(signature)
        lines.append("{")

        if func_node.body:
            body_lines = self._generate_block(func_node.body)
            lines.extend(body_lines)

        lines.append("}")

        logger.debug(f"Code generation complete for {func_node.name}")

        return lines

    def _generate_function_signature(self, func_node: FunctionNode) -> str:
        return_type = func_node.return_type or "void"
        name = func_node.name

        params = ", ".join(
            f"{p.var_type or 'object'} {p.var_name}" for p in func_node.parameters
        )

        return (
            f"{'public ' if func_node.is_public else ''}{return_type} {name}({params})"
        )

    def _generate_block(self, block: BlockNode) -> list[str]:
        lines = []

        for stmt in block.statements:
            stmt_lines = self._generate_statement(stmt)
            lines.extend(stmt_lines)

        return lines

    def _generate_statement(self, stmt: StatementNode) -> list[str]:
        match stmt:
            case VariableDeclNode():
                return self._generate_variable_decl(stmt)
            case AssignmentNode():
                return self._generate_assignment(stmt)
            case ExpressionStatementNode():
                return self._generate_expression_statement(stmt)
            case IfNode():
                return self._generate_if(stmt)
            case IfElseNode():
                return self._generate_if_else(stmt)
            case WhileNode():
                return self._generate_while(stmt)
            case DoWhileNode():
                return self._generate_do_while(stmt)
            case SwitchNode():
                return self._generate_switch(stmt)
            case LabelNode():
                return [f"{stmt.label_name}:"]
            case GotoNode():
                return [f"goto {stmt.target_label};"]
            case BreakNode():
                return ["break;"]
            case ContinueNode():
                return ["continue;"]
            case ReturnNode():
                return ["return;"]
            case StatementNode():
                raise Exception("Unexpected raw StatementNode!")

    def _generate_variable_decl(self, stmt: VariableDeclNode) -> list[str]:
        var_type = stmt.var_type or "object"

        if stmt.initial_value:
            value = self._generate_expression(stmt.initial_value)
            return [f"{var_type} {stmt.var_name} = {value};"]
        else:
            return [f"{var_type} {stmt.var_name};"]

    def _generate_assignment(self, stmt: AssignmentNode) -> list[str]:
        if stmt.value:
            value = self._generate_expression(stmt.value)
            return [f"{stmt.target} = {value};"]
        else:
            return [f"{stmt.target} = <unknown>;"]

    def _generate_expression_statement(
        self, stmt: ExpressionStatementNode
    ) -> list[str]:
        if stmt.expression:
            expr_str = self._generate_expression(stmt.expression)
            return [f"{expr_str};"]
        else:
            return ["/* empty expression */;"]

    def _generate_if(self, stmt: IfNode) -> list[str]:
        lines = []

        condition = (
            self._generate_expression(stmt.condition) if stmt.condition else "false"
        )
        lines.append(f"if ({condition})")
        lines.append("{")

        if stmt.then_block:
            lines.extend(self._generate_block(stmt.then_block))

        lines.append("}")

        return lines

    def _generate_if_else(self, stmt: IfElseNode) -> list[str]:
        lines = []

        condition_expr = stmt.condition
        then_block = stmt.then_block
        else_block = stmt.else_block

        if not self._block_has_statements(then_block) and self._block_has_statements(
            else_block
        ):
            condition_expr = self._invert_condition(condition_expr)
            then_block, else_block = else_block, then_block

        condition = (
            self._generate_expression(condition_expr) if condition_expr else "false"
        )
        lines.append(f"if ({condition})")
        lines.append("{")

        if then_block:
            lines.extend(self._generate_block(then_block))

        lines.append("}")

        if else_block is not None:
            if self._block_has_statements(else_block):
                if self._is_single_if_statement_block(else_block):
                    nested_stmt = else_block.statements[0]
                    nested_lines = self._generate_statement(nested_stmt)
                    if nested_lines:
                        first_line = nested_lines[0]
                        if first_line.startswith("if "):
                            lines.append(f"else {first_line}")
                            lines.extend(nested_lines[1:])
                            return lines
                    lines.append("else")
                    lines.append("{")
                    lines.extend(nested_lines)
                    lines.append("}")
                    return lines

                lines.append("else")
                lines.append("{")
                lines.extend(self._generate_block(else_block))
                lines.append("}")

        return lines

    def _generate_while(self, stmt: WhileNode) -> list[str]:
        lines = []

        condition = (
            self._generate_expression(stmt.condition) if stmt.condition else "true"
        )
        lines.append(f"while ({condition})")
        lines.append("{")

        if stmt.body:
            lines.extend(self._generate_block(stmt.body))

        lines.append("}")

        return lines

    def _generate_do_while(self, stmt: DoWhileNode) -> list[str]:
        lines = []

        lines.append("do")
        lines.append("{")

        if stmt.body:
            lines.extend(self._generate_block(stmt.body))

        lines.append("}")

        condition = (
            self._generate_expression(stmt.condition) if stmt.condition else "true"
        )
        lines.append(f"while ({condition});")

        return lines

    def _generate_switch(self, stmt: SwitchNode) -> list[str]:
        lines = []

        expr_str = (
            self._generate_expression(stmt.expression)
            if stmt.expression
            else "<switch>"
        )
        lines.append(f"switch ({expr_str})")
        lines.append("{")

        for case in stmt.cases:
            lines.extend(self._generate_switch_case(case))

        if stmt.default_case:
            lines.extend(self._generate_switch_case(stmt.default_case))

        lines.append("}")

        return lines

    def _generate_switch_case(self, stmt: SwitchCaseNode) -> list[str]:
        lines = []

        if stmt.is_default:
            lines.append("default:")
        else:
            for value in stmt.values:
                value_str = self._generate_expression(value)
                lines.append(f"case {value_str}:")

        if stmt.body:
            lines.extend(self._generate_block(stmt.body))

        if not self._switch_case_terminates(stmt):
            lines.append("break;")

        return lines

    def _switch_case_terminates(self, stmt: SwitchCaseNode) -> bool:
        if not stmt.body or not stmt.body.statements:
            return False
        last_stmt = stmt.body.statements[-1]
        return isinstance(last_stmt, (ReturnNode, GotoNode, BreakNode, ContinueNode))

    def _generate_expression(self, expr: ExpressionNode) -> str:
        match expr:
            case LiteralNode():
                return self._generate_literal(expr)
            case VariableNode():
                return expr.var_name
            case CallNode():
                return self._generate_call(expr)
            case PropertyAccessNode():
                return self._generate_property_access(expr)
            case ConstructionNode():
                return self._generate_construction(expr)
            case OperatorNode():
                return self._generate_operator(expr)
            case TypeNode():
                return expr.type_name
            case _:
                return f"<{expr.expr_type}>"

    def _generate_literal(self, expr: LiteralNode) -> str:
        match expr.literal_type:
            case "System.String":
                if expr.value is None:
                    return "null"
                return json.dumps(str(expr.value))
            case "System.Boolean":
                return "true" if expr.value else "false"
            case "System.Int32" | "System.Int64":
                return str(expr.value)
            case "System.Single" | "System.Double":
                return (
                    f"{expr.value}f"
                    if expr.literal_type == "System.Single"
                    else str(expr.value)
                )
            case _:
                if expr.value is None:
                    return "null"
                else:
                    return f"null /* {json.dumps(expr.value)} */"

    def _generate_call(self, expr: CallNode) -> str:
        if not expr.is_external:
            return f"{expr.function_name}()"

        args = list(expr.arguments)
        if expr.is_static:
            caller = expr.type_name
        else:
            _, param_expr = args.pop(0)
            caller = self._generate_expression(param_expr)
        func_name = expr.original_name
        receiver = None
        if not expr.returns_void and not expr.emit_as_expression:
            receiver_expr = expr.receiver
            if receiver_expr is None:
                raise Exception(
                    "Invalid CallNode! Returns non-void but receiver not detected!"
                )
            receiver = self._generate_expression(receiver_expr)

        args_str = ", ".join(
            f"{'out ' if arg_type != ParameterType.IN else ''}"
            f"{self._generate_expression(arg)}"
            for arg_type, arg in args
        )

        provider_str = f"{caller}.{func_name or expr.function_name}({args_str})"
        if expr.returns_void or expr.emit_as_expression or receiver is None:
            return provider_str
        return f"{receiver} = {provider_str}"

    def _generate_property_access(self, expr: PropertyAccessNode) -> str:
        if expr.is_static:
            this = expr.type_name
        else:
            if expr.this is None:
                raise Exception(
                    "Invalid PropertyAccessNode! Non-static but expr.this is None!"
                )
            this = self._generate_expression(expr.this)

        prop = f"{this}.{expr.field}"

        match expr.access_type:
            case PropertyAccessType.GET:
                if expr.emit_as_expression:
                    return prop
                target = (
                    self._generate_expression(expr.target)
                    if expr.target is not None
                    else "<unknown>"
                )
                return f"{target} = {prop}"
            case PropertyAccessType.SET:
                value = (
                    self._generate_expression(expr.value)
                    if expr.value is not None
                    else "<unknown>"
                )
                return f"{prop} = {value}"

    def _generate_construction(self, expr: ConstructionNode) -> str:
        args = ", ".join(self._generate_expression(arg) for arg in expr.arguments)
        if expr.emit_as_expression or expr.receiver is None:
            return f"new {expr.type_name}({args})"
        receiver = self._generate_expression(expr.receiver)
        return f"{receiver} = new {expr.type_name}({args})"

    def _generate_operator(self, expr: OperatorNode) -> str:
        if expr.operator == Operator.Conversion:
            if len(expr.operands) != 2:
                raise Exception("Invalid implicit conversion operands!")
            type_str = self._generate_expression(expr.operands[0])
            value_str = self._generate_operator_operand(
                expr.operator, expr.operands[1], 1
            )
            oprs = [type_str, value_str]
        else:
            oprs = [
                self._generate_operator_operand(expr.operator, opr, i)
                for i, opr in enumerate(expr.operands)
            ]
        if expr.emit_as_expression or expr.receiver is None:
            return expr.operator.formatter.format(*oprs)
        receiver = self._generate_expression(expr.receiver)
        return f"{receiver} = {expr.operator.formatter.format(*oprs)}"


class ProgramCodeGenerator:
    # todo: this may disturbe test
    _class_counter: int = 0
    _generator: CSharpCodeGenerator = CSharpCodeGenerator()

    @classmethod
    def generate_program(
        cls,
        program: UdonProgramData,
        function_analyzers: dict[str, FunctionDataFlowAnalyzer],
    ) -> tuple[Optional[str], str]:
        function_nodes = []

        for func_name, analyzer in function_analyzers.items():
            logger.debug(f"Generating function: {func_name}")

            from .ast_builder import ASTBuilder

            ast_builder = ASTBuilder(analyzer.program, analyzer)
            func_node = ast_builder.build()
            function_nodes.append(func_node)

        referenced_variable_names = cls._collect_referenced_variable_names(
            function_nodes
        )
        global_vars = cls._collect_and_generate_global_variables(
            function_analyzers, referenced_variable_names
        )

        program_node = ProgramNode(
            global_variables=global_vars, functions=function_nodes
        )

        class_name = program.get_class_name()
        namespace = None
        name_fallback = False
        if class_name is None:
            cls._class_counter += 1
            name_fallback = True
            class_name = f"DecompiledClass_{cls._class_counter}"
        elif "." in class_name:
            namespace, class_name = class_name.rsplit(".", 1)
            if not namespace:
                namespace = None

        program_node.namespace = namespace

        code = cls._generator.generate(program_node, class_name)

        logger.debug("Program code generation complete")

        return class_name if not name_fallback else None, code

    @staticmethod
    def _collect_and_generate_global_variables(
        function_analyzers: dict[str, FunctionDataFlowAnalyzer],
        referenced_variable_names: Optional[set[str]] = None,
    ) -> list[VariableDeclNode]:
        from udon_decompiler.analysis.variable_identifier import VariableScope

        logger.debug("Collecting global variables from all functions...")

        global_vars_by_address: dict[int, Variable] = {}

        for func_name, analyzer in function_analyzers.items():
            for var in analyzer.variables.values():
                if var.scope == VariableScope.GLOBAL:
                    if var.address not in global_vars_by_address:
                        global_vars_by_address[var.address] = var
                        logger.debug(f"Found global variable in {func_name}: {var}")

        res = []

        if not function_analyzers:
            return res

        first_analyzer = next(iter(function_analyzers.values()))
        program = first_analyzer.program

        for var in global_vars_by_address.values():
            initial_heap_value = program.get_initial_heap_value(var.address)
            if ProgramCodeGenerator._is_hidden_global_variable(
                var, initial_heap_value, referenced_variable_names
            ):
                continue

            initial_value = ProgramCodeGenerator._format_initial_value(
                var, initial_heap_value
            )

            res.append(
                VariableDeclNode(
                    var_name=var.name,
                    var_type=var.type_hint or "object",
                    initial_value=LiteralNode(
                        value=initial_value, literal_type=var.type_hint
                    ),
                )
            )

        return res

    @staticmethod
    def _is_hidden_global_variable(
        var: Variable,
        initial_heap_value: Optional[HeapEntry],
        referenced_variable_names: Optional[set[str]] = None,
    ) -> bool:
        symbol_name = var.original_symbol.name if var.original_symbol else var.name

        if symbol_name in {UdonProgramData.CLASS_NAME_SYMBOL_NAME, "__refl_typeid"}:
            return True

        if symbol_name.startswith(SymbolInfo.CONST_SYMBOL_PREFIX):
            if initial_heap_value is None or initial_heap_value.value is None:
                return False
            return initial_heap_value.value.is_serializable

        if symbol_name.startswith(SymbolInfo.INTERNAL_SYMBOL_PREFIX):
            return True

        if symbol_name.startswith(SymbolInfo.GLOBAL_INTERNAL_SYMBOL_PREFIX):
            if referenced_variable_names is None:
                return True
            return var.name not in referenced_variable_names

        if symbol_name.startswith(SymbolInfo.THIS_SYMBOL_PREFIX):
            return True

        return False

    @staticmethod
    def _collect_referenced_variable_names(
        function_nodes: list[FunctionNode],
    ) -> set[str]:
        referenced: set[str] = set()
        for func in function_nodes:
            if func.body:
                ProgramCodeGenerator._collect_from_block(func.body, referenced)
        return referenced

    @staticmethod
    def _collect_from_block(block: BlockNode, referenced: set[str]) -> None:
        for stmt in block.statements:
            ProgramCodeGenerator._collect_from_statement(stmt, referenced)

    @staticmethod
    def _collect_from_statement(stmt: StatementNode, referenced: set[str]) -> None:
        match stmt:
            case AssignmentNode(target=target, value=value):
                if target and not (target.startswith("<") and target.endswith(">")):
                    referenced.add(target)
                if value:
                    ProgramCodeGenerator._collect_from_expression(value, referenced)
            case ExpressionStatementNode(expression=expression):
                if expression:
                    ProgramCodeGenerator._collect_from_expression(
                        expression, referenced
                    )
            case IfNode(condition=condition, then_block=then_block):
                if condition:
                    ProgramCodeGenerator._collect_from_expression(condition, referenced)
                if then_block:
                    ProgramCodeGenerator._collect_from_block(then_block, referenced)
            case IfElseNode(
                condition=condition, then_block=then_block, else_block=else_block
            ):
                if condition:
                    ProgramCodeGenerator._collect_from_expression(condition, referenced)
                if then_block:
                    ProgramCodeGenerator._collect_from_block(then_block, referenced)
                if else_block:
                    ProgramCodeGenerator._collect_from_block(else_block, referenced)
            case WhileNode(condition=condition, body=body):
                if condition:
                    ProgramCodeGenerator._collect_from_expression(condition, referenced)
                if body:
                    ProgramCodeGenerator._collect_from_block(body, referenced)
            case DoWhileNode(condition=condition, body=body):
                if condition:
                    ProgramCodeGenerator._collect_from_expression(condition, referenced)
                if body:
                    ProgramCodeGenerator._collect_from_block(body, referenced)
            case SwitchNode(
                expression=expression, cases=cases, default_case=default_case
            ):
                if expression:
                    ProgramCodeGenerator._collect_from_expression(
                        expression, referenced
                    )
                for case in cases:
                    ProgramCodeGenerator._collect_from_case(case, referenced)
                if default_case:
                    ProgramCodeGenerator._collect_from_case(default_case, referenced)
            case _:
                return

    @staticmethod
    def _collect_from_case(case: SwitchCaseNode, referenced: set[str]) -> None:
        for value in case.values:
            ProgramCodeGenerator._collect_from_expression(value, referenced)
        if case.body:
            ProgramCodeGenerator._collect_from_block(case.body, referenced)

    @staticmethod
    def _collect_from_expression(expr: ExpressionNode, referenced: set[str]) -> None:
        match expr:
            case VariableNode(var_name=var_name):
                # finally an exit
                if var_name:
                    referenced.add(var_name)
            case LiteralNode():
                return
            case CallNode(receiver=receiver, arguments=arguments):
                if receiver:
                    ProgramCodeGenerator._collect_from_expression(receiver, referenced)
                for _, arg in arguments:
                    ProgramCodeGenerator._collect_from_expression(arg, referenced)
            case PropertyAccessNode(this=this, target=target, value=value):
                if this:
                    ProgramCodeGenerator._collect_from_expression(this, referenced)
                if target:
                    ProgramCodeGenerator._collect_from_expression(target, referenced)
                if value:
                    ProgramCodeGenerator._collect_from_expression(value, referenced)
            case OperatorNode(receiver=receiver, operands=operands):
                if receiver:
                    ProgramCodeGenerator._collect_from_expression(receiver, referenced)
                for operand in operands:
                    ProgramCodeGenerator._collect_from_expression(operand, referenced)
            case ConstructionNode(receiver=receiver, arguments=arguments):
                if receiver:
                    ProgramCodeGenerator._collect_from_expression(receiver, referenced)
                for arg in arguments:
                    ProgramCodeGenerator._collect_from_expression(arg, referenced)
            case TypeNode():
                return
            case _:
                raise Exception("Invalid raw expression!")

    @staticmethod
    def _format_initial_value(var: Variable, initial_heap_value: Optional[HeapEntry]):
        if initial_heap_value is None or initial_heap_value.value is None:
            return "null"
        if initial_heap_value.value.is_serializable:
            return initial_heap_value.value.value
        else:
            value = initial_heap_value.value.value
            if isinstance(value, dict):
                to_string = value.get("toString")
                if isinstance(to_string, str):
                    return to_string
            return "<non-serializable>"
