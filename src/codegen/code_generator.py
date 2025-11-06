from typing import Optional
from .ast_nodes import *
from .formatter import CodeFormatter
from ..utils.logger import logger


class CSharpCodeGenerator:
    def __init__(self, use_formatting: bool = True):
        self.formatter = CodeFormatter() if use_formatting else None

    def generate(self, func_node: FunctionNode) -> str:
        logger.info(f"Generating C# code for {func_node.name}...")

        lines = []

        signature = self._generate_function_signature(func_node)
        lines.append(signature)
        lines.append("{")

        if func_node.body:
            body_lines = self._generate_block(func_node.body, indent=1)
            lines.extend(body_lines)

        lines.append("}")

        code = "\n".join(lines)

        if self.formatter:
            code = self.formatter.format(code)

        logger.info(f"Code generation complete for {func_node.name}")

        return code

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
    def __init__(self):
        self.function_generator = CSharpCodeGenerator()

    def generate_program(
        self,
        function_analyzers: dict,
        class_name: str = "UdonBehavior"
    ) -> str:
        logger.info(f"Generating C# code for entire program...")

        lines = []

        lines.extend([
            "// Decompiled Udon Program",
            "// This is pseudo-code and may not compile directly",
            "",
            "using UnityEngine;",
            "using VRC.SDKBase;",
            "using VRC.Udon;",
            "",
            f"public class {class_name} : UdonSharpBehaviour",
            "{",
        ])

        for func_name, analyzer in function_analyzers.items():
            logger.info(f"Generating function: {func_name}")

            from .ast_builder import ASTBuilder
            ast_builder = ASTBuilder(analyzer.program, analyzer)
            func_node = ast_builder.build()

            func_code = self.function_generator.generate(func_node)

            func_lines = func_code.split("\n")
            indented_lines = ["    " + line for line in func_lines]

            lines.append("")
            lines.extend(indented_lines)

        lines.append("}")

        code = "\n".join(lines)

        logger.info("Program code generation complete")

        return code
