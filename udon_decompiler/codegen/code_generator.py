from typing import Optional

from udon_decompiler.models.program import UdonProgramData
from udon_decompiler.analysis.variable_identifier import Variable
from udon_decompiler.analysis.dataflow_analyzer import FunctionDataFlowAnalyzer
from udon_decompiler.codegen.ast_nodes import *
from udon_decompiler.codegen.formatter import CodeFormatter
from udon_decompiler.utils.logger import logger


class CSharpCodeGenerator:
    def __init__(self, use_formatting: bool = True):
        self.formatter = CodeFormatter() if use_formatting else None

    def generate(self, program_node: ProgramNode, class_name: str) -> str:
        logger.info(f"Generating C# code for {class_name}...")

        lines = []

        lines.extend([
            "// Decompiled Udon Program",
            "// This is pseudo-code and may not compile directly",
            "",
            f"public class {class_name} : UdonSharpBehaviour",
            "{",
        ])

        for global_var in program_node.global_variables:
            decl_lines = self._generate_variable_decl(global_var, 1)
            lines.extend(decl_lines)

        lines.append("\n")

        for func_node in program_node.functions:
            func_lines = self._generate_function(func_node, indent=1)
            lines.extend(func_lines)

        lines.append("}")

        code = "\n".join(lines)

        if self.formatter:
            code = self.formatter.format(code)

        logger.info(f"Code generation complete for {func_node.name}")

        return code

    def _generate_function(self, func_node: FunctionNode, indent: int = 0) -> list[str]:
        logger.info(f"Generating C# code for {func_node.name}...")

        indent_str = "    " * indent

        lines = []

        signature = indent_str + self._generate_function_signature(func_node)
        lines.append(signature)
        lines.append(f"{indent_str}{{")

        if func_node.body:
            body_lines = self._generate_block(
                func_node.body, indent=indent + 1)
            lines.extend(body_lines)

        lines.append(f"{indent_str}}}")

        logger.info(f"Code generation complete for {func_node.name}")

        return lines

    def _generate_function_signature(self, func_node: FunctionNode) -> str:
        return_type = func_node.return_type or "void"
        name = func_node.name

        params = ", ".join(
            f"{p.var_type or 'object'} {p.var_name}"
            for p in func_node.parameters
        )

        return f"public {return_type} {name}({params})"

    def _generate_block(self, block: BlockNode, indent: int = 0) -> list[str]:
        lines = []

        for stmt in block.statements:
            stmt_lines = self._generate_statement(stmt, indent)
            lines.extend(stmt_lines)

        return lines

    def _generate_statement(self, stmt: StatementNode, indent: int = 0) -> list[str]:
        indent_str = "    " * indent

        if isinstance(stmt, VariableDeclNode):
            return self._generate_variable_decl(stmt, indent)

        elif isinstance(stmt, AssignmentNode):
            return self._generate_assignment(stmt, indent)

        elif isinstance(stmt, ExpressionStatementNode):
            return self._generate_expression_statement(stmt, indent)

        elif isinstance(stmt, IfNode):
            return self._generate_if(stmt, indent)

        elif isinstance(stmt, IfElseNode):
            return self._generate_if_else(stmt, indent)

        elif isinstance(stmt, WhileNode):
            return self._generate_while(stmt, indent)

        elif isinstance(stmt, DoWhileNode):
            return self._generate_do_while(stmt, indent)

        elif isinstance(stmt, LabelNode):
            return [f"{indent_str}{stmt.label_name}:"]

        elif isinstance(stmt, GotoNode):
            return [f"{indent_str}goto {stmt.target_label};"]

        else:
            return [f"{indent_str}// Unknown statement type: {stmt.node_type.value}"]

    def _generate_variable_decl(self, stmt: VariableDeclNode, indent: int) -> list[str]:
        indent_str = "    " * indent
        var_type = stmt.var_type or "object"

        if stmt.initial_value:
            value = self._generate_expression(stmt.initial_value)
            return [f"{indent_str}{var_type} {stmt.var_name} = {value};"]
        else:
            return [f"{indent_str}{var_type} {stmt.var_name};"]

    def _generate_assignment(self, stmt: AssignmentNode, indent: int) -> list[str]:
        indent_str = "    " * indent

        if stmt.value:
            value = self._generate_expression(stmt.value)
            return [f"{indent_str}{stmt.target} = {value};"]
        else:
            return [f"{indent_str}{stmt.target} = <unknown>;"]

    def _generate_expression_statement(self, stmt: ExpressionStatementNode, indent: int) -> list[str]:
        indent_str = "    " * indent

        if stmt.expression:
            expr_str = self._generate_expression(stmt.expression)
            return [f"{indent_str}{expr_str};"]
        else:
            return [f"{indent_str}/* empty expression */;"]

    def _generate_if(self, stmt: IfNode, indent: int) -> list[str]:
        indent_str = "    " * indent
        lines = []

        condition = self._generate_expression(
            stmt.condition) if stmt.condition else "false"
        lines.append(f"{indent_str}if ({condition})")
        lines.append(f"{indent_str}{{")

        if stmt.then_block:
            lines.extend(self._generate_block(stmt.then_block, indent + 1))

        lines.append(f"{indent_str}}}")

        return lines

    def _generate_if_else(self, stmt: IfElseNode, indent: int) -> list[str]:
        indent_str = "    " * indent
        lines = []

        condition = self._generate_expression(
            stmt.condition) if stmt.condition else "false"
        lines.append(f"{indent_str}if ({condition})")
        lines.append(f"{indent_str}{{")

        if stmt.then_block:
            lines.extend(self._generate_block(stmt.then_block, indent + 1))

        lines.append(f"{indent_str}}}")
        lines.append(f"{indent_str}else")
        lines.append(f"{indent_str}{{")

        if stmt.else_block:
            lines.extend(self._generate_block(stmt.else_block, indent + 1))

        lines.append(f"{indent_str}}}")

        return lines

    def _generate_while(self, stmt: WhileNode, indent: int) -> list[str]:
        indent_str = "    " * indent
        lines = []

        condition = self._generate_expression(
            stmt.condition) if stmt.condition else "true"
        lines.append(f"{indent_str}while ({condition})")
        lines.append(f"{indent_str}{{")

        if stmt.body:
            lines.extend(self._generate_block(stmt.body, indent + 1))

        lines.append(f"{indent_str}}}")

        return lines

    def _generate_do_while(self, stmt: DoWhileNode, indent: int) -> list[str]:
        indent_str = "    " * indent
        lines = []

        lines.append(f"{indent_str}do")
        lines.append(f"{indent_str}{{")

        if stmt.body:
            lines.extend(self._generate_block(stmt.body, indent + 1))

        lines.append(f"{indent_str}}}")

        condition = self._generate_expression(
            stmt.condition) if stmt.condition else "true"
        lines.append(f"{indent_str}while ({condition});")

        return lines

    def _generate_expression(self, expr: ExpressionNode) -> str:
        if isinstance(expr, LiteralNode):
            return self._generate_literal(expr)

        elif isinstance(expr, VariableNode):
            return expr.var_name

        elif isinstance(expr, CallNode):
            return self._generate_call(expr)

        else:
            return f"<{expr.expr_type}>"

    def _generate_literal(self, expr: LiteralNode) -> str:

        if expr.literal_type == "System.String":
            value = str(expr.value).replace('"', '\\"')
            return f'"{value}"'

        elif expr.literal_type == "System.Boolean":
            return "true" if expr.value else "false"

        elif expr.literal_type == "System.Int32" or expr.literal_type == "System.Int64":
            return str(expr.value)

        elif expr.literal_type == "System.Single" or expr.literal_type == "System.Double":
            return f"{expr.value}f" if expr.literal_type == "System.Single" else str(expr.value)

        elif expr.value is None:
            return "null"

        else:
            return str(expr.value)

    def _generate_call(self, expr: CallNode) -> str:
        func_name = expr.function_signature

        args = ", ".join(self._generate_expression(arg)
                         for arg in expr.arguments)

        return f"{func_name}({args})"


class ProgramCodeGenerator:
    _class_counter: int = 0
    _generator: CSharpCodeGenerator = CSharpCodeGenerator()

    @classmethod
    def generate_program(
        cls,
        program: UdonProgramData,
        function_analyzers: dict[str, FunctionDataFlowAnalyzer],
    ) -> tuple[Optional[str], str]:
        global_vars = cls._collect_and_generate_global_variables(
            function_analyzers)

        program_node = ProgramNode(global_variables=global_vars)

        for func_name, analyzer in function_analyzers.items():
            logger.info(f"Generating function: {func_name}")

            from .ast_builder import ASTBuilder
            ast_builder = ASTBuilder(analyzer.program, analyzer)
            func_node = ast_builder.build()

            program_node.functions.append(func_node)

        class_name = program.get_class_name()
        name_fallback = False
        if not class_name:
            cls._class_counter += 1
            name_fallback = True
            class_name = f"DecompiledClass_{cls._class_counter}"

        code = cls._generator.generate(program_node, class_name)

        logger.info("Program code generation complete")

        return class_name if not name_fallback else None, code

    @staticmethod
    def _collect_and_generate_global_variables(function_analyzers: dict[str, FunctionDataFlowAnalyzer]) -> list[VariableDeclNode]:
        from udon_decompiler.analysis.variable_identifier import VariableScope

        logger.info("Collecting global variables from all functions...")

        global_vars_by_address: dict[int, Variable] = {}

        for func_name, analyzer in function_analyzers.items():
            for var in analyzer.variables.values():
                if var.scope == VariableScope.GLOBAL:
                    if var.address not in global_vars_by_address:
                        global_vars_by_address[var.address] = var
                        logger.debug(
                            f"Found global variable in {func_name}: {var}")

        res = []

        if not function_analyzers:
            return res

        first_analyzer = next(iter(function_analyzers.values()))
        program = first_analyzer.program

        for var in global_vars_by_address.values():
            initial_heap_value = program.get_initial_heap_value(var.address)
            if initial_heap_value is None or initial_heap_value.value is None:
                initial_value = "null"
            elif not initial_heap_value.value.is_serializable:
                initial_value = "<non-serializable>"
            else:
                initial_value = initial_heap_value.value.value

            res.append(
                VariableDeclNode(
                    var_name=var.name,
                    var_type=var.type_hint or "object",
                    initial_value=LiteralNode(
                        value=initial_value,
                        literal_type=var.type_hint
                    )
                )
            )

        return res
