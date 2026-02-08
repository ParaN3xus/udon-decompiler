import json
import shutil
import subprocess
from dataclasses import dataclass
from typing import Optional, Set

from udon_decompiler.analysis.dataflow_analyzer import FunctionDataFlowAnalyzer
from udon_decompiler.analysis.expression_builder import Operator
from udon_decompiler.analysis.ir import (
    AssignmentStatement,
    ConditionalTerminator,
    ConstructorCallExpression,
    EndTerminator,
    ExpressionStatement,
    ExternalCallExpression,
    FunctionIR,
    GotoTerminator,
    InternalCallExpression,
    IRExpression,
    LiteralExpression,
    OperatorCallExpression,
    PropertyAccessExpression,
    ReturnTerminator,
    SwitchTerminator,
    VariableExpression,
)
from udon_decompiler.analysis.ir.nodes import ClassIR, GlobalVariableIR
from udon_decompiler.analysis.variable_identifier import Variable
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

    def generate(self, class_ir: ClassIR) -> str:
        logger.info(f"Generating C# code for {class_ir.class_name}...")

        lines = [
            "// Decompiled Udon Program",
            "// This is pseudo-code and may not compile directly",
            "",
        ]

        class_lines = [f"public class {class_ir.class_name} : UdonSharpBehaviour", "{"]

        for global_var in class_ir.global_variables:
            class_lines.append(self._generate_global_variable(global_var))

        if class_ir.global_variables:
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

    def _generate_global_variable(self, global_var: GlobalVariableIR) -> str:
        var_type = global_var.variable.type_hint or "object"
        initial = self._generate_literal(
            value=global_var.initial_value,
            literal_type=global_var.variable.type_hint,
        )
        return f"{var_type} {global_var.variable.name} = {initial};"

    def _generate_function(self, function: FunctionIR) -> list[str]:
        function_signature = (
            f"{'public ' if function.is_function_public else ''}"
            f"void {function.function_name}()"
        )
        lines = [
            function_signature,
            "{",
        ]

        for variable in function.variable_declarations:
            lines.append(f"{variable.type_hint or 'object'} {variable.name};")

        if function.variable_declarations:
            lines.append("")

        referenced_labels = self._collect_referenced_labels(function)
        for index, block_start in enumerate(function.block_order):
            block = function.blocks[block_start]
            next_start = (
                function.block_order[index + 1]
                if index + 1 < len(function.block_order)
                else None
            )

            if block_start in referenced_labels:
                lines.append(f"{self._label_for(block_start)}:")

            for statement in block.statements:
                lines.append(self._generate_statement(statement))

            lines.extend(self._generate_terminator(block.terminator, next_start))

        lines.append("}")
        return lines

    def _collect_referenced_labels(self, function: FunctionIR) -> Set[int]:
        labels: Set[int] = set()
        for index, block_start in enumerate(function.block_order):
            block = function.blocks[block_start]
            next_start = (
                function.block_order[index + 1]
                if index + 1 < len(function.block_order)
                else None
            )
            labels.update(self._terminator_goto_targets(block.terminator, next_start))
        return labels

    def _terminator_goto_targets(
        self, terminator, next_start: Optional[int]
    ) -> Set[int]:
        targets: Set[int] = set()
        match terminator:
            case GotoTerminator(target=target):
                if target != next_start:
                    targets.add(target)
            case ConditionalTerminator(
                true_target=true_target, false_target=false_target
            ):
                if false_target == next_start:
                    if true_target != next_start:
                        targets.add(true_target)
                elif true_target == next_start:
                    targets.add(false_target)
                else:
                    targets.add(true_target)
                    targets.add(false_target)
            case SwitchTerminator(switch_targets=switch_targets):
                for target in switch_targets:
                    if target != next_start:
                        targets.add(target)
            case ReturnTerminator() | EndTerminator():
                pass
            case _:
                raise Exception(
                    f"Unsupported terminator type: {type(terminator).__name__}"
                )
        return targets

    def _generate_statement(self, statement) -> str:
        match statement:
            case AssignmentStatement(target=target, value=value):
                value_str = self._generate_expression(value, as_value=True)
                return f"{target.name} = {value_str};"
            case ExpressionStatement(expression=expression):
                return f"{self._generate_expression(expression, as_value=False)};"
            case _:
                raise Exception(
                    f"Unsupported IR statement type: {type(statement).__name__}"
                )

    def _generate_terminator(self, terminator, next_start: Optional[int]) -> list[str]:
        match terminator:
            case GotoTerminator(target=target):
                if target == next_start:
                    return []
                return [f"goto {self._label_for(target)};"]
            case ConditionalTerminator(
                condition=condition,
                true_target=true_target,
                false_target=false_target,
            ):
                return self._generate_conditional_terminator(
                    condition=condition,
                    true_target=true_target,
                    false_target=false_target,
                    next_start=next_start,
                )
            case SwitchTerminator(
                switch_index=switch_index, switch_targets=switch_targets
            ):
                return self._generate_switch_terminator(
                    switch_index=switch_index,
                    switch_targets=switch_targets,
                    next_start=next_start,
                )
            case ReturnTerminator():
                return ["return;"]
            case EndTerminator():
                return []
            case _:
                raise Exception(
                    f"Unsupported terminator type: {type(terminator).__name__}"
                )

    def _generate_conditional_terminator(
        self,
        condition: IRExpression,
        true_target: int,
        false_target: int,
        next_start: Optional[int],
    ) -> list[str]:
        condition_str = self._generate_expression(condition, as_value=True)
        if false_target == next_start:
            if true_target == next_start:
                return []
            return self._if_goto(condition_str, self._label_for(true_target))

        if true_target == next_start:
            return self._if_goto(f"!({condition_str})", self._label_for(false_target))

        lines = self._if_goto(condition_str, self._label_for(true_target))
        lines.append(f"goto {self._label_for(false_target)};")
        return lines

    def _generate_switch_terminator(
        self,
        switch_index: IRExpression,
        switch_targets: list[int],
        next_start: Optional[int],
    ) -> list[str]:
        lines: list[str] = []
        index_str = self._generate_expression(switch_index, as_value=True)
        for index, target in enumerate(switch_targets):
            if target == next_start:
                continue
            lines.extend(
                self._if_goto(f"{index_str} == {index}", self._label_for(target))
            )
        return lines

    def _if_goto(self, condition: str, label: str) -> list[str]:
        return [f"if ({condition})", "{", f"goto {label};", "}"]

    def _generate_expression(self, expression: IRExpression, as_value: bool) -> str:
        match expression:
            case LiteralExpression(value=value, type_hint=type_hint):
                return self._generate_literal(value=value, literal_type=type_hint)
            case VariableExpression(variable=variable):
                return variable.name
            case InternalCallExpression(entry_point=entry_point):
                function_name = entry_point.name or (
                    f"func_0x{entry_point.call_jump_target:08x}"
                )
                return f"{function_name}()"
            case ExternalCallExpression(
                function_info=function_info,
                arguments=arguments,
            ):
                return self._generate_external_call(
                    function_info=function_info,
                    arguments=arguments,
                )
            case PropertyAccessExpression(
                function_info=function_info,
                arguments=arguments,
            ):
                return self._generate_property_access(
                    function_info=function_info,
                    arguments=arguments,
                    as_value=as_value,
                )
            case ConstructorCallExpression(
                function_info=function_info,
                arguments=arguments,
            ):
                args = ", ".join(
                    self._generate_expression(argument, as_value=True)
                    for argument in arguments
                )
                return f"new {function_info.type_name}({args})"
            case OperatorCallExpression(
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
        if not isinstance(child_expression, OperatorCallExpression):
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
                return "true" if value else "false"
            case "System.Int32" | "System.Int64" | "System.UInt32":
                return str(value)
            case "System.Single":
                return f"{value}f"
            case "System.Double":
                return str(value)
            case _:
                if value is None:
                    return "null"
                return f"null /* {json.dumps(value)} */"

    def _label_for(self, block_start: int) -> str:
        return f"L_{block_start:08x}"

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
