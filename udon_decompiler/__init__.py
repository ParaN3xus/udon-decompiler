from typing import Optional

from udon_decompiler.analysis import (
    BasicBlock,
    BasicBlockIdentifier,
    BasicBlockType,
    CFGBuilder,
    ControlFlowGraph,
    DataFlowAnalyzer,
    FunctionDataFlowAnalyzer,
    IRAssignmentStatement,
    IRBuilder,
    IRConstructorCallExpression,
    IRExpression,
    IRExpressionStatement,
    IRExternalCallExpression,
    IRFunction,
    IRInternalCallExpression,
    IRLiteralExpression,
    IROperatorCallExpression,
    IRPropertyAccessExpression,
    IRStatement,
    IRVariableExpression,
    StackFrame,
    StackSimulator,
    StackValue,
    Variable,
    VariableIdentifier,
    VariableScope,
)
from udon_decompiler.codegen import (
    CSharpCodeGenerator,
    IRClass,
)
from udon_decompiler.codegen.program_code_generator import ProgramCodeGenerator
from udon_decompiler.loaders import ModuleInfoLoader, ProgramLoader
from udon_decompiler.models import (
    EntryPointInfo,
    ExternFunctionInfo,
    HeapEntry,
    HeapEntryValue,
    Instruction,
    OpCode,
    ParameterType,
    SymbolInfo,
    UdonModuleInfo,
    UdonProgramData,
)
from udon_decompiler.parsers import BytecodeParser
from udon_decompiler.utils import logger, setup_logger


def decompile_program_to_source(program: UdonProgramData) -> tuple[Optional[str], str]:
    bc_parser = BytecodeParser(program)
    instructions = bc_parser.parse()

    logger.debug(f"ASM: {instructions}")

    analyzer = DataFlowAnalyzer(program, instructions)
    ir_class = analyzer.analyze()

    class_name, code = ProgramCodeGenerator.generate_program(ir_class)
    return class_name, code


__all__ = [
    "BasicBlock",
    "BasicBlockIdentifier",
    "BasicBlockType",
    "CFGBuilder",
    "ControlFlowGraph",
    "DataFlowAnalyzer",
    "FunctionDataFlowAnalyzer",
    "StackFrame",
    "IRBuilder",
    "IRExpression",
    "IRLiteralExpression",
    "IRVariableExpression",
    "IRInternalCallExpression",
    "IRExternalCallExpression",
    "IRPropertyAccessExpression",
    "IRConstructorCallExpression",
    "IROperatorCallExpression",
    "IRStatement",
    "IRAssignmentStatement",
    "IRExpressionStatement",
    "IRFunction",
    "StackSimulator",
    "StackValue",
    "Variable",
    "VariableIdentifier",
    "VariableScope",
    "IRClass",
    "CSharpCodeGenerator",
    "ProgramCodeGenerator",
    "ModuleInfoLoader",
    "ProgramLoader",
    "EntryPointInfo",
    "ExternFunctionInfo",
    "HeapEntry",
    "HeapEntryValue",
    "Instruction",
    "OpCode",
    "ParameterType",
    "SymbolInfo",
    "UdonModuleInfo",
    "UdonProgramData",
    "BytecodeParser",
    "logger",
    "setup_logger",
    "decompile_program_to_source",
]
