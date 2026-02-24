import json
import shutil
import subprocess
from typing import Optional, Set

from udon_decompiler.analysis.expression_builder import Operator
from udon_decompiler.analysis.ir.nodes import (
    IRAssignmentStatement,
    IRBlock,
    IRClass,
    IRConstructorCallExpression,
    IRExpression,
    IRExpressionStatement,
    IRExternalCallExpression,
    IRFunction,
    IRIf,
    IRInternalCallExpression,
    IRLiteralExpression,
    IROperatorCallExpression,
    IRPropertyAccessExpression,
    IRStatement,
    IRVariableDeclearationStatement,
    IRVariableExpression,
)
from udon_decompiler.models.module_info import ParameterType
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

    def generate(self, class_ir: IRClass) -> str:
        logger.info(f"Generating C# code for {class_ir.class_name}...")

        lines = [
            "// Decompiled Udon Program",
            "// This is pseudo-code and may not compile directly",
            "",
        ]

        class_lines = [f"public class {class_ir.class_name} : UdonSharpBehaviour", "{"]

        for global_var in class_ir.variable_declarations:
            class_lines.append(self._generate_variable_declaration(global_var))

        if class_ir.variable_declarations:
            class_lines.append("")

        for function in class_ir.functions:
            class_lines.extend(self._generate_function(function))
            class_lines.append("")

        if class_ir.functions:
            class_lines.pop()

        class_lines.append("}")

        if class_ir.namespace:
            lines.extend([f"namespace {class_ir.namespace}", "{"])
            lines.extend(class_lines)
            lines.append("}")
        else:
            lines.extend(class_lines)

        code = "\n".join(lines)
        return self._format(code)

    def _generate_variable_declaration(
        self, declaration: IRVariableDeclearationStatement
    ) -> str:
        variable = declaration.variable
        var_type = variable.type_hint or "object"
        if declaration.init_value is None:
            return f"{var_type} {variable.name};"

        initial = self._generate_expression(declaration.init_value, as_value=True)
        return f"{var_type} {variable.name} = {initial};"

    def _generate_function(self, function: IRFunction) -> list[str]:
        function_signature = (
            f"{'public ' if function.is_function_public else ''}"
            f"void {function.function_name}()"
        )
        lines = [
            function_signature,
            "{",
        ]

        for declaration in function.variable_declarations:
            lines.append(self._generate_variable_declaration(declaration))

        if function.variable_declarations:
            lines.append("")

        if function.body is None:
            raise Exception("Function body is None")
        # lines.extend(self._generate_block_container(function.body))

        lines.append("}")
        return lines

    def _generate_statement_lines(self, statement: IRStatement) -> list[str]:
        match statement:
            case IRAssignmentStatement(target=target, value=value):
                value_str = self._generate_expression(value, as_value=True)
                return [f"{target.name} = {value_str};"]
            case IRExpressionStatement(expression=expression):
                return [f"{self._generate_expression(expression, as_value=False)};"]
            case IRVariableDeclearationStatement() as declaration:
                return [self._generate_variable_declaration(declaration)]
            case IRIf(
                condition=condition,
                true_statement=true_statement,
                false_statement=false_statement,
            ):
                condition_str = self._generate_expression(condition, as_value=True)
                lines = [f"if ({condition_str})", "{"]
                lines.extend(self._generate_statement_lines(true_statement))
                lines.append("}")
                if false_statement is not None:
                    lines.extend(["else", "{"])
                    lines.extend(self._generate_statement_lines(false_statement))
                    lines.append("}")
                return lines
            # case IRSwitchStatement(
            #     test_value=test_value, cases=cases, default_body=default_body
            # ):
            #     lines = [
            #        f"switch ({self._generate_expression(test_value, as_value=True)})",
            #         "{",
            #     ]
            #     for switch_case in cases:
            #         for case_value in switch_case.values:
            #             rendered_case = self._generate_expression(
            #                 case_value, as_value=True
            #             )
            #             lines.append(f"case {rendered_case}:")
            #         lines.extend(self._generate_block_contents(switch_case.body))
            #         lines.append("break;")
            #     if default_body is not None:
            #         lines.append("default:")
            #         lines.extend(self._generate_block_contents(default_body))
            #         lines.append("break;")
            #     lines.append("}")
            #     return lines
            # case IRWhileLoopStatement(condition=condition, body=body):
            #     condition_str = self._generate_expression(condition, as_value=True)
            #     lines = [f"while ({condition_str})", "{"]
            #     lines.extend(self._generate_block_contents(body))
            #     lines.append("}")
            #     return lines
            # case IRDoWhileLoopStatement(condition=condition, body=body):
            #     condition_str = self._generate_expression(condition, as_value=True)
            #     lines = ["do", "{"]
            #     lines.extend(self._generate_block_contents(body))
            #     lines.append(f"}} while ({condition_str});")
            #     return lines
            # case IRBreakStatement():
            #     return ["break;"]
            # case IRContinueStatement():
            #     return ["continue;"]
            # case IRTerminator() as terminator:
            #     return self._generate_terminator(terminator, None)
            # case IRReturnStatement():
            #     return ["return;"]
            case IRBlock() as block:
                return self._generate_block_contents(block)
            # case IRBasicBlock() as block:
            #    lines = []
            #    if block.should_emit_label:
            #        lines.append(f"{self._label_for(block)}:")
            #
            #    for stmt in block.statements:
            #        lines.extend(self._generate_statement_lines(stmt))
            #
            #    if block.terminator:
            #        lines.extend(self._generate_statement_lines(block.terminator))
            #
            #    return lines
            case _:
                raise Exception(
                    f"Unsupported IR statement type: {type(statement).__name__}"
                )

    def _generate_block_contents(self, block: IRBlock) -> list[str]:
        lines: list[str] = []
        for statement in block.statements:
            lines.extend(self._generate_statement_lines(statement))
        return lines

    @staticmethod
    def _label_for_id(block_id: int) -> str:
        return f"label_bb_{block_id:08x}"

    # def _generate_terminator(
    #     self, terminator: IRTerminator, next_block: Optional[IRBasicBlock]
    # ) -> list[str]:
    #     match terminator:
    #         case IRJumpTerminator(target=target):
    #             if self._is_fallthrough(target, next_block):
    #                 return []
    #             return [f"goto {self._label_for(target)};"]
    #         case IRConditionalJumpTerminator(
    #             condition=condition,
    #             true_target=true_target,
    #             false_target=false_target,
    #         ):
    #             return self._generate_conditional_terminator(
    #                 condition=condition,
    #                 true_target=true_target,
    #                 false_target=false_target,
    #                 next_block=next_block,
    #             )
    #         case IRSwitchTerminator(
    #             index_expression=index_expression,
    #             cases=cases,
    #             default_target=default_target,
    #         ):
    #             return self._generate_switch_terminator(
    #                 index_expression=index_expression,
    #                 cases=cases,
    #                 default_target=default_target,
    #                 next_block=next_block,
    #             )
    #         case IRReturnTerminator():
    #             return ["return;"]
    #         case IRNoOpTerminator():
    #             return []
    #         case _:
    #             raise Exception(
    #                 f"Unsupported terminator type: {type(terminator).__name__}"
    #             )

    # def _generate_conditional_terminator(
    #    self,
    #    condition: IRExpression,
    #    true_target: IRBasicBlock,
    #    false_target: IRBasicBlock,
    #    next_block: Optional[IRBasicBlock],
    # ) -> list[str]:
    #    condition_str = self._generate_expression(condition, as_value=True)
    #
    #    true_is_next = self._is_fallthrough(true_target, next_block)
    #    false_is_next = self._is_fallthrough(false_target, next_block)
    #
    #    if false_is_next:
    #        if true_is_next:
    #            return []
    #        return self._if_goto(condition_str, self._label_for(true_target))
    #
    #    if true_is_next:
    #        return self._if_goto(
    #            f"!({condition_str})",
    #            self._label_for(false_target),
    #        )
    #
    #    lines = self._if_goto(condition_str, self._label_for(true_target))
    #    lines.append(f"goto {self._label_for(false_target)};")
    #    return lines
    #
    # def _generate_switch_terminator(
    #    self,
    #    index_expression: IRExpression,
    #    cases: dict[int, IRBasicBlock],
    #    default_target: Optional[IRBasicBlock],
    #    next_block: Optional[IRBasicBlock],
    # ) -> list[str]:
    #    lines: list[str] = []
    #    index_str = self._generate_expression(index_expression, as_value=True)
    #
    #    for case_value, target in sorted(cases.items(), key=lambda item: item[0]):
    #        if self._is_fallthrough(target, next_block):
    #            continue
    #        lines.extend(
    #            self._if_goto(
    #                f"{index_str} == {case_value}",
    #                self._label_for(target),
    #            )
    #        )
    #
    #    if default_target is not None and not self._is_fallthrough(
    #        default_target,
    #        next_block,
    #    ):
    #        lines.append(f"goto {self._label_for(default_target)};")
    #
    #    return lines

    def _if_goto(self, condition: str, label: str) -> list[str]:
        return [f"if ({condition})", "{", f"goto {label};", "}"]

    # def _is_fallthrough(
    #    self,
    #    candidate: IRBasicBlock,
    #    next_block: Optional[IRBasicBlock],
    # ) -> bool:
    #    return next_block is not None and candidate.id == next_block.id

    def _generate_expression(self, expression: IRExpression, as_value: bool) -> str:
        match expression:
            case IRLiteralExpression(value=value, type_hint=type_hint):
                return self._generate_literal(value=value, literal_type=type_hint)
            case IRVariableExpression(variable=variable):
                return variable.name
            case IRInternalCallExpression(entry_point=entry_point):
                function_name = entry_point.name or (
                    f"func_0x{entry_point.call_jump_target:08x}"
                )
                return f"{function_name}()"
            case IRExternalCallExpression(
                function_info=function_info,
                arguments=arguments,
            ):
                return self._generate_external_call(
                    function_info=function_info,
                    arguments=arguments,
                )
            case IRPropertyAccessExpression(
                function_info=function_info,
                arguments=arguments,
            ):
                return self._generate_property_access(
                    function_info=function_info,
                    arguments=arguments,
                    as_value=as_value,
                )
            case IRConstructorCallExpression(
                function_info=function_info,
                arguments=arguments,
            ):
                args = ", ".join(
                    self._generate_expression(argument, as_value=True)
                    for argument in arguments
                )
                return f"new {function_info.type_name}({args})"
            case IROperatorCallExpression(
                operator=operator,
                function_info=function_info,
                arguments=arguments,
            ):
                return self._generate_operator(
                    operator=operator,
                    function_info=function_info,
                    arguments=arguments,
                )
            case _:
                raise Exception(
                    f"Unsupported IR expression type: {type(expression).__name__}"
                )

    def _generate_external_call(
        self, function_info, arguments: list[IRExpression]
    ) -> str:
        if function_info.is_static:
            caller = function_info.type_name
            argument_start = 0
        else:
            if not arguments:
                raise Exception(
                    f"Instance call has no receiver: {function_info.signature}"
                )
            caller = self._generate_expression(arguments[0], as_value=True)
            argument_start = 1

        call_arguments = arguments[argument_start:]
        if argument_start + len(call_arguments) > len(function_info.parameters):
            raise Exception(
                f"Argument count exceeds metadata for {function_info.signature}"
            )

        rendered_arguments: list[str] = []
        for index, argument in enumerate(call_arguments):
            parameter_type = function_info.parameters[argument_start + index]
            argument_str = self._generate_expression(argument, as_value=True)
            if parameter_type != ParameterType.IN:
                argument_str = f"out {argument_str}"
            rendered_arguments.append(argument_str)

        function_name = function_info.original_name or function_info.function_name
        args_str = ", ".join(rendered_arguments)
        return f"{caller}.{function_name}({args_str})"

    def _generate_property_access(
        self, function_info, arguments: list[IRExpression], as_value: bool
    ) -> str:
        function_name = function_info.function_name
        prefix_len = 5
        if len(function_name) < prefix_len:
            raise Exception(f"Invalid property function name: {function_name}")

        access = function_name[:prefix_len]
        if access not in {"__get", "__set"}:
            raise Exception(f"Unsupported property access type: {function_name}")

        field_name = function_info.original_name or function_info.function_name

        if function_info.is_static:
            owner = function_info.type_name
            value_index = 0
        else:
            if not arguments:
                raise Exception(
                    "Instance property access has no receiver: "
                    f"{function_info.signature}"
                )
            owner = self._generate_expression(arguments[0], as_value=True)
            value_index = 1

        if access == "__get":
            if (function_info.is_static and len(arguments) != 0) or (
                not function_info.is_static and len(arguments) != 1
            ):
                raise Exception(
                    f"Invalid property getter arguments for {function_info.signature}"
                )
            return f"{owner}.{field_name}"

        if len(arguments) <= value_index:
            raise Exception(
                f"Property setter missing value argument: {function_info.signature}"
            )

        value = self._generate_expression(arguments[value_index], as_value=True)
        if as_value:
            return f"{owner}.{field_name} = {value}"
        return f"{owner}.{field_name} = {value}"

    def _generate_operator(
        self, operator, function_info, arguments: list[IRExpression]
    ) -> str:
        if operator == Operator.Conversion:
            if len(arguments) != 1:
                raise Exception(
                    f"Conversion operator expects 1 arg, got {len(arguments)}"
                )
            value = self._generate_operator_operand(
                parent_op=operator,
                operand=arguments[0],
                operand_index=0,
            )
            return f"({function_info.type_name}){value}"

        operands = [
            self._generate_operator_operand(
                parent_op=operator,
                operand=operand,
                operand_index=index,
            )
            for index, operand in enumerate(arguments)
        ]
        return operator.formatter.format(*operands)

    def _generate_operator_operand(
        self, parent_op: Operator, operand: IRExpression, operand_index: int
    ) -> str:
        operand_str = self._generate_expression(operand, as_value=True)
        if self._operator_needs_parentheses(parent_op, operand, operand_index):
            return f"({operand_str})"
        return operand_str

    def _operator_needs_parentheses(
        self, parent_op: Operator, child_expression: IRExpression, operand_index: int
    ) -> bool:
        if not isinstance(child_expression, IROperatorCallExpression):
            return False

        child_op = child_expression.operator
        parent_prec = self._operator_precedence(parent_op)
        child_prec = self._operator_precedence(child_op)

        if child_prec < parent_prec:
            return True
        if child_prec > parent_prec:
            return False

        if parent_op in self._UNARY_OPERATORS:
            return True

        if operand_index == 1:
            if child_op != parent_op:
                return True
            return parent_op not in self._ASSOCIATIVE_OPERATORS

        return False

    def _operator_precedence(self, op: Operator) -> int:
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

    def _generate_literal(self, value: object, literal_type: Optional[str]) -> str:
        match literal_type:
            case "System.String":
                if value is None:
                    return "null"
                return json.dumps(str(value))
            case "System.Boolean":
                return "true" if bool(value) else "false"
            case "System.Int32" | "System.Int64" | "System.UInt32":
                if value is None:
                    return "0"
                return str(value)
            case "System.Single":
                if value is None:
                    return "0f"
                return f"{value}f"
            case "System.Double":
                if value is None:
                    return "0d"
                return str(value)
            case _:
                if value is None:
                    return "null"
                return f"null /* {self._safe_json(value)} */"

    def _safe_json(self, value: object) -> str:
        try:
            return json.dumps(value)
        except TypeError:
            return json.dumps(str(value))

    # def _label_for(self, block: IRBasicBlock) -> str:
    #     return f"label_bb_{block.id:08x}"

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
