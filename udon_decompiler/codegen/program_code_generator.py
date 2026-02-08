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
from udon_decompiler.codegen.code_generator import CSharpCodeGenerator
from udon_decompiler.models.module_info import ParameterType
from udon_decompiler.models.program import HeapEntry, SymbolInfo, UdonProgramData
from udon_decompiler.utils.logger import logger


class ProgramCodeGenerator:
    _class_counter: int = 0
    _generator: CSharpCodeGenerator = CSharpCodeGenerator()

    @classmethod
    def generate_program(
        cls,
        program: UdonProgramData,
        function_analyzers: dict[str, FunctionDataFlowAnalyzer],
    ) -> tuple[Optional[str], str]:
        functions = [analyzer.get_ir() for analyzer in function_analyzers.values()]

        referenced_variable_names = cls._collect_referenced_variable_names(functions)
        global_variables = cls._collect_global_variables(
            function_analyzers=function_analyzers,
            referenced_variable_names=referenced_variable_names,
        )

        class_name = program.get_class_name()
        namespace = None
        name_fallback = False
        if class_name is None:
            logger.warning(
                "Failed to identify class name, using synthetic fallback name."
            )
            cls._class_counter += 1
            name_fallback = True
            class_name = f"DecompiledClass_{cls._class_counter}"
        elif "." in class_name:
            namespace, class_name = class_name.rsplit(".", 1)
            if not namespace:
                namespace = None

        class_ir = ClassIR(
            class_name=class_name,
            namespace=namespace,
            program=program,
            global_variables=global_variables,
            functions=functions,
        )
        code = cls._generator.generate(class_ir)
        return class_name if not name_fallback else None, code

    @staticmethod
    def _collect_global_variables(
        function_analyzers: dict[str, FunctionDataFlowAnalyzer],
        referenced_variable_names: Optional[set[str]],
    ) -> list[GlobalVariableIR]:
        from udon_decompiler.analysis.variable_identifier import VariableScope

        global_vars_by_address: dict[int, Variable] = {}
        for analyzer in function_analyzers.values():
            for variable in analyzer.variables.values():
                if variable.scope != VariableScope.GLOBAL:
                    continue
                global_vars_by_address.setdefault(variable.address, variable)

        if not function_analyzers:
            return []

        program = next(iter(function_analyzers.values())).program
        result: list[GlobalVariableIR] = []
        for variable in sorted(
            global_vars_by_address.values(), key=lambda v: v.address
        ):
            initial_heap_value = program.get_initial_heap_value(variable.address)
            if ProgramCodeGenerator._is_hidden_global_variable(
                variable,
                initial_heap_value,
                referenced_variable_names,
            ):
                continue
            initial_value = ProgramCodeGenerator._format_initial_value(
                variable, initial_heap_value
            )
            result.append(
                GlobalVariableIR(variable=variable, initial_value=initial_value)
            )
        return result

    @staticmethod
    def _collect_referenced_variable_names(functions: list[FunctionIR]) -> set[str]:
        referenced: set[str] = set()
        for function in functions:
            for block_start in function.block_order:
                block = function.blocks[block_start]
                for statement in block.statements:
                    match statement:
                        case AssignmentStatement(target=target, value=value):
                            referenced.add(target.name)
                            ProgramCodeGenerator._collect_from_expression(
                                value, referenced
                            )
                        case ExpressionStatement(expression=expression):
                            ProgramCodeGenerator._collect_from_expression(
                                expression, referenced
                            )
                        case _:
                            raise Exception(
                                "Unsupported IR statement type while collecting refs: "
                                f"{type(statement).__name__}"
                            )

                match block.terminator:
                    case ConditionalTerminator(condition=condition):
                        ProgramCodeGenerator._collect_from_expression(
                            condition, referenced
                        )
                    case SwitchTerminator(switch_index=switch_index):
                        ProgramCodeGenerator._collect_from_expression(
                            switch_index, referenced
                        )
                    case GotoTerminator() | ReturnTerminator() | EndTerminator():
                        pass
                    case _:
                        raise Exception(
                            "Unsupported IR terminator type while collecting refs: "
                            f"{type(block.terminator).__name__}"
                        )

        return referenced

    @staticmethod
    def _collect_from_expression(expr: IRExpression, referenced: set[str]) -> None:
        match expr:
            case VariableExpression(variable=variable):
                referenced.add(variable.name)
            case LiteralExpression():
                return
            case InternalCallExpression():
                return
            case (
                ExternalCallExpression(arguments=arguments)
                | PropertyAccessExpression(arguments=arguments)
                | ConstructorCallExpression(arguments=arguments)
                | OperatorCallExpression(arguments=arguments)
            ):
                for argument in arguments:
                    ProgramCodeGenerator._collect_from_expression(argument, referenced)
            case _:
                raise Exception(
                    "Unsupported IR expression type while collecting refs: "
                    f"{type(expr).__name__}"
                )

    @staticmethod
    def _is_hidden_global_variable(
        variable: Variable,
        initial_heap_value: Optional[HeapEntry],
        referenced_variable_names: Optional[set[str]],
    ) -> bool:
        symbol_name = (
            variable.original_symbol.name if variable.original_symbol else variable.name
        )

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
            return variable.name not in referenced_variable_names

        if symbol_name.startswith(SymbolInfo.THIS_SYMBOL_PREFIX):
            return True

        return False

    @staticmethod
    def _format_initial_value(
        variable: Variable, initial_heap_value: Optional[HeapEntry]
    ):
        if initial_heap_value is None or initial_heap_value.value is None:
            return None
        if initial_heap_value.value.is_serializable:
            return initial_heap_value.value.value

        value = initial_heap_value.value.value
        if isinstance(value, dict):
            to_string = value.get("toString")
            if isinstance(to_string, str):
                return to_string
        return None
