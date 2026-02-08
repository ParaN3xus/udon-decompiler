from typing import Optional

from udon_decompiler.analysis import (
    AssignmentStatement,
    BasicBlock,
    BasicBlockIdentifier,
    BasicBlockType,
    BlockIR,
    CFGBuilder,
    ConditionalTerminator,
    ConstructorCallExpression,
    ControlFlowGraph,
    DataFlowAnalyzer,
    EndTerminator,
    Expression,
    ExpressionBuilder,
    ExpressionStatement,
    ExpressionType,
    ExternalCallExpression,
    FunctionDataFlowAnalyzer,
    FunctionIR,
    GotoTerminator,
    InternalCallExpression,
    IRBuilder,
    IRExpression,
    IRStatement,
    IRTerminator,
    LiteralExpression,
    OperatorCallExpression,
    PropertyAccessExpression,
    ReturnTerminator,
    StackFrame,
    StackSimulator,
    StackValue,
    SwitchTerminator,
    Variable,
    VariableExpression,
    VariableIdentifier,
    VariableScope,
)
from udon_decompiler.codegen import (
    ClassIR,
    CSharpCodeGenerator,
    GlobalVariableIR,
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
    function_analyzers = analyzer.analyze()

    class_name, code = ProgramCodeGenerator.generate_program(
        program, function_analyzers
    )
    return class_name, code


__all__ = [
    "BasicBlock",
    "BasicBlockIdentifier",
    "BasicBlockType",
    "CFGBuilder",
    "ControlFlowGraph",
    "DataFlowAnalyzer",
    "Expression",
    "ExpressionBuilder",
    "ExpressionType",
    "FunctionDataFlowAnalyzer",
    "StackFrame",
    "IRBuilder",
    "IRExpression",
    "LiteralExpression",
    "VariableExpression",
    "InternalCallExpression",
    "ExternalCallExpression",
    "PropertyAccessExpression",
    "ConstructorCallExpression",
    "OperatorCallExpression",
    "IRStatement",
    "AssignmentStatement",
    "ExpressionStatement",
    "IRTerminator",
    "GotoTerminator",
    "ConditionalTerminator",
    "SwitchTerminator",
    "ReturnTerminator",
    "EndTerminator",
    "BlockIR",
    "FunctionIR",
    "StackSimulator",
    "StackValue",
    "Variable",
    "VariableIdentifier",
    "VariableScope",
    "ClassIR",
    "CSharpCodeGenerator",
    "GlobalVariableIR",
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
