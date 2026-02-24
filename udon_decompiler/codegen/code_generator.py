import json
import shutil
import subprocess
from typing import Optional

from clang_format import get_executable

from udon_decompiler.analysis.ir.nodes import (
    IRAssignmentStatement,
    IRBlock,
    IRBlockContainer,
    IRClass,
    IRConstructorCallExpression,
    IRContainerKind,
    IRExpression,
    IRExpressionStatement,
    IRExternalCallExpression,
    IRFunction,
    IRHighLevelDoWhile,
    IRHighLevelSwitch,
    IRHighLevelWhile,
    IRIf,
    IRInternalCallExpression,
    IRJump,
    IRLeave,
    IRLiteralExpression,
    IROperatorCallExpression,
    IRPropertyAccessExpression,
    IRReturn,
    IRStatement,
    IRSwitch,
    IRVariableDeclearationStatement,
    IRVariableExpression,
)
from udon_decompiler.analysis.operator import Operator
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

    def __init__(self) -> None:
        self._function_body: Optional[IRBlockContainer] = None
        self._synthetic_label_counter = 0
        self._block_label_cache: dict[int, str] = {}
        self._container_exit_labels: dict[int, str] = {}

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
        self._function_body = function.body
        self._synthetic_label_counter = 0
        self._block_label_cache = {}
        self._container_exit_labels = {}

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

        lines.extend(
            self._generate_block_container(
                function.body,
                is_root=True,
                break_target=None,
                continue_target=None,
            )
        )

        lines.append("}")
        return lines

    def _generate_block_container(
        self,
        container: IRBlockContainer,
        is_root: bool,
        break_target: Optional[IRBlockContainer],
        continue_target: Optional[IRBlock],
    ) -> list[str]:
        if container.kind == IRContainerKind.SWITCH:
            raise Exception(
                "Low-level switch container reached codegen; "
                "expected HighLevelSwitchTransform to lift it first."
            )

        return self._generate_unstructured_container(
            container=container,
            is_root=is_root,
            break_target=break_target,
            continue_target=continue_target,
        )

    def _generate_unstructured_container(
        self,
        container: IRBlockContainer,
        is_root: bool,
        break_target: Optional[IRBlockContainer],
        continue_target: Optional[IRBlock],
    ) -> list[str]:
        lines: list[str] = []
        if not is_root:
            lines.append("{")

        for index, block in enumerate(container.blocks):
            if block.should_emit_label:
                lines.append(f"{self._label_for_block(block)}:")
            next_block = (
                container.blocks[index + 1]
                if index + 1 < len(container.blocks)
                else None
            )
            for stmt_index, statement in enumerate(block.statements):
                statement_next_block = (
                    next_block if stmt_index == len(block.statements) - 1 else None
                )
                lines.extend(
                    self._generate_statement(
                        statement=statement,
                        current_container=container,
                        next_block=statement_next_block,
                        break_target=break_target,
                        continue_target=continue_target,
                    )
                )

        if container.should_emit_exit_label:
            exit_label = self._get_container_exit_label(container)
            lines.append(f"{exit_label}:")
            lines.append(";")

        if not is_root:
            lines.append("}")
        return lines

    def _generate_statement(
        self,
        statement: IRStatement,
        current_container: Optional[IRBlockContainer],
        next_block: Optional[IRBlock],
        break_target: Optional[IRBlockContainer],
        continue_target: Optional[IRBlock],
    ) -> list[str]:
        if isinstance(statement, IRAssignmentStatement):
            rhs = self._generate_expression(statement.value, as_value=True)
            return [f"{statement.target.name} = {rhs};"]

        if isinstance(statement, IRExpressionStatement):
            expression = self._generate_expression(
                statement.expression,
                as_value=False,
            )
            return [f"{expression};"]

        if isinstance(statement, IRIf):
            return self._generate_if_statement(
                statement=statement,
                current_container=current_container,
                break_target=break_target,
                continue_target=continue_target,
            )

        if isinstance(statement, IRHighLevelWhile):
            return self._generate_high_level_while(statement)

        if isinstance(statement, IRHighLevelDoWhile):
            return self._generate_high_level_do_while(statement)

        if isinstance(statement, IRHighLevelSwitch):
            return self._generate_high_level_switch(
                statement=statement,
                current_container=current_container,
            )

        if isinstance(statement, IRJump):
            if continue_target is not None and statement.target is continue_target:
                return ["continue;"]
            if next_block is not None and statement.target is next_block:
                return []
            return [f"goto {self._label_for_block(statement.target)};"]

        if isinstance(statement, IRLeave):
            if (
                self._function_body is not None
                and statement.target_container is self._function_body
            ):
                return ["return;"]
            if break_target is not None and statement.target_container is break_target:
                return ["break;"]
            exit_label = self._get_container_exit_label(statement.target_container)
            return [f"goto {exit_label};"]

        if isinstance(statement, IRReturn):
            return ["return;"]

        if isinstance(statement, IRSwitch):
            raise Exception(
                "Low-level IRSwitch reached codegen; "
                "expected HighLevelSwitchTransform to lift it first."
            )

        if isinstance(statement, IRBlock):
            lines: list[str] = []
            for nested in statement.statements:
                lines.extend(
                    self._generate_statement(
                        statement=nested,
                        current_container=current_container,
                        next_block=None,
                        break_target=break_target,
                        continue_target=continue_target,
                    )
                )
            return lines

        if isinstance(statement, IRBlockContainer):
            return self._generate_block_container(
                container=statement,
                is_root=False,
                break_target=break_target,
                continue_target=continue_target,
            )

        raise Exception(f"Unsupported IR statement type: {type(statement).__name__}")

    def _generate_if_statement(
        self,
        statement: IRIf,
        current_container: Optional[IRBlockContainer],
        break_target: Optional[IRBlockContainer],
        continue_target: Optional[IRBlock],
    ) -> list[str]:
        condition = self._generate_expression(statement.condition, as_value=True)
        lines = [f"if ({condition})", "{"]
        lines.extend(
            self._generate_branch_statement(
                statement.true_statement,
                current_container=current_container,
                break_target=break_target,
                continue_target=continue_target,
            )
        )
        lines.append("}")

        if statement.false_statement is not None:
            lines.append("else")
            lines.append("{")
            lines.extend(
                self._generate_branch_statement(
                    statement.false_statement,
                    current_container=current_container,
                    break_target=break_target,
                    continue_target=continue_target,
                )
            )
            lines.append("}")

        return lines

    def _generate_branch_statement(
        self,
        statement: IRStatement,
        current_container: Optional[IRBlockContainer],
        break_target: Optional[IRBlockContainer],
        continue_target: Optional[IRBlock],
    ) -> list[str]:
        if isinstance(statement, IRBlock):
            if current_container is not None and statement in current_container.blocks:
                return [f"goto {self._label_for_block(statement)};"]

            lines: list[str] = []
            if statement.should_emit_label:
                lines.append(f"{self._label_for_block(statement)}:")
            for nested in statement.statements:
                lines.extend(
                    self._generate_statement(
                        statement=nested,
                        current_container=current_container,
                        next_block=None,
                        break_target=break_target,
                        continue_target=continue_target,
                    )
                )
            return lines

        return self._generate_statement(
            statement=statement,
            current_container=current_container,
            next_block=None,
            break_target=break_target,
            continue_target=continue_target,
        )

    def _generate_high_level_switch(
        self,
        statement: IRHighLevelSwitch,
        current_container: Optional[IRBlockContainer],
    ) -> list[str]:
        index_expr = self._generate_expression(
            statement.index_expression,
            as_value=True,
        )
        lines = [f"switch ({index_expr})", "{"]

        for section in statement.sections:
            for label in section.labels:
                lines.append(f"case {label}:")
            if section.is_default:
                lines.append("default:")

            for nested in section.body.statements:
                lines.extend(
                    self._generate_statement(
                        statement=nested,
                        current_container=current_container,
                        next_block=None,
                        break_target=None,
                        continue_target=None,
                    )
                )

            if self._switch_section_needs_break(section.body):
                lines.append("break;")

        lines.append("}")
        return lines

    def _generate_high_level_while(
        self,
        statement: IRHighLevelWhile,
    ) -> list[str]:
        condition = (
            "true"
            if statement.condition is None
            else self._generate_expression(statement.condition, as_value=True)
        )
        lines = [f"while ({condition})", "{"]
        lines.extend(
            self._generate_block_container(
                container=statement.body,
                is_root=True,
                break_target=statement.break_target,
                continue_target=statement.continue_target,
            )
        )
        lines.append("}")
        return lines

    def _generate_high_level_do_while(
        self,
        statement: IRHighLevelDoWhile,
    ) -> list[str]:
        lines = ["do", "{"]
        lines.extend(
            self._generate_block_container(
                container=statement.body,
                is_root=True,
                break_target=statement.break_target,
                continue_target=statement.continue_target,
            )
        )
        condition = self._generate_expression(statement.condition, as_value=True)
        lines.append(f"}} while ({condition});")
        return lines

    @staticmethod
    def _switch_section_needs_break(body: IRBlock) -> bool:
        if not body.statements:
            return True
        last = body.statements[-1]
        return not isinstance(last, (IRJump, IRLeave, IRReturn))

    def _label_for_block(self, block: IRBlock) -> str:
        if block.start_address >= 0:
            return f"label_bb_{block.start_address:08x}"

        key = id(block)
        existing = self._block_label_cache.get(key)
        if existing is not None:
            return existing

        label = f"label_synth_{self._synthetic_label_counter:04d}"
        self._synthetic_label_counter += 1
        self._block_label_cache[key] = label
        return label

    def _get_container_exit_label(self, container: IRBlockContainer) -> str:
        key = id(container)
        existing = self._container_exit_labels.get(key)
        if existing is not None:
            return existing

        label = f"label_exit_{len(self._container_exit_labels):04d}"
        self._container_exit_labels[key] = label
        return label

    # region Expression
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
                arguments=arguments,
            ):
                return self._generate_operator(
                    operator=operator,
                    function_info=None,
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
            if function_info is None:
                return value
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

    # endregion

    def _format(self, code: str) -> str:
        clang_format = get_executable("clang-format")
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
