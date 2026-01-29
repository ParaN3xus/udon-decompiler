from typing import Optional

from udon_decompiler.analysis import (
    BasicBlock,
    BasicBlockIdentifier,
    BasicBlockType,
    CFGBuilder,
    ControlFlowGraph,
    DataFlowAnalyzer,
    Expression,
    ExpressionBuilder,
    ExpressionType,
    FunctionDataFlowAnalyzer,
    StackFrame,
    StackSimulator,
    StackValue,
    Variable,
    VariableIdentifier,
    VariableScope,
)
from udon_decompiler.codegen import (
    AssignmentNode,
    ASTBuilder,
    ASTNode,
    BlockNode,
    CallNode,
    CSharpCodeGenerator,
    DoWhileNode,
    ExpressionNode,
    ExpressionStatementNode,
    FunctionNode,
    IfElseNode,
    LiteralNode,
    ProgramCodeGenerator,
    ProgramNode,
    StatementNode,
    VariableDeclNode,
    VariableNode,
    WhileNode,
)
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
    "StackSimulator",
    "StackValue",
    "Variable",
    "VariableIdentifier",
    "VariableScope",
    "AssignmentNode",
    "ASTBuilder",
    "ASTNode",
    "BlockNode",
    "CallNode",
    "CSharpCodeGenerator",
    "DoWhileNode",
    "ExpressionNode",
    "ExpressionStatementNode",
    "FunctionNode",
    "IfElseNode",
    "LiteralNode",
    "ProgramCodeGenerator",
    "ProgramNode",
    "StatementNode",
    "VariableDeclNode",
    "VariableNode",
    "WhileNode",
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
