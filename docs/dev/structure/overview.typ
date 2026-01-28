#import "/docs/book.typ": book-page
#import "@preview/shiroa:0.3.1": shiroa-sys-target
#import "../../_utils/common.typ": cross-link-heading

#show: book-page.with(title: "概览")

本节介绍本项目代码仓库的文件结构和本项目的大致工作流程.

= 代码仓库
```shell-unix-generic
$ tree -L 2 --gitignore
.
├── docs                                      # 你正在阅读的文档
│   ├── about
│   ├── book.typ
│   ├── dev
│   ├── introduction.typ
│   ├── user
│   └── _utils
├── Editor                                    # 编辑器脚本
│   ├── UdonModuleInfoExtractor.cs
│   ├── UdonProgramDumper.cs
│   ├── UdonProgramDumperGUI.cs
│   ├── UdonSharpSourceTextCompilerCLI.cs
│   ├── UdonSharpSourceTextCompiler.cs
│   └── UdonSharpSourceTextCompilerGUI.cs
├── LICENSE
├── pyproject.toml
├── README.md
├── tests                                     # 测试系统和测试样例
│   ├── cases
│   ├── ci
│   ├── __init__.py
│   └── test_snapshots.py
├── udon_decompiler                           # 反编译器源代码
│   ├── analysis
│   ├── codegen
│   ├── __init__.py
│   ├── loaders
│   ├── __main__.py
│   ├── models
│   ├── parsers
│   └── utils
└── uv.lock
```

= 工作流程
== 解析 `serializedProgramCompressedBytes`
由于本反编译器使用 Python 编写, 而 Python 并没有 `OdinSerializer` 的反序列化器可用, 所以这一步骤使用编辑器脚本 `UdonProgramDumper.cs` 完成.

基本上, 这个脚本的功能就是把序列化的 Udon Program 反序列化, 然后重新序列化为一个对 Python 更友好的 `.json` 文件.

== 反编译

这部分内容的入口点在 `decompile_program_to_source` 函数中. 按照嵌套层次, 可以分为如下步骤
- 反汇编: `BytecodeParser`
- 数据流分析: `DataFlowAnalyzer`
  - 构建 CFG: `CFGBuilder`
    - 识别入口点并划分基本块: `._identify_entry_points()` 和 `._identify_hidden_entry_points()`
    - 构建边: `._build_edges()`
    - 构建每个函数的 CFG: `._build_function_cfgs()`
      - 识别函数的所有基本块: `._find_function_blocks()`
      - 识别函数名: `_identify_function_name()`
      - 根据 `._build_edges()` 的结果构建实际的函数 CFG
  - 函数数据流分析: `FunctionDataFlowAnalyzer`
    - 栈模拟: `._simulate_stack()` 和 `StackSimulator`
    - 识别变量: `VariableIdentifier`
    - 构建表达式: `ExpressionBuilder`
- 代码生成: `ProgramCodeGenerator`
  - 构建每个函数的 AST: `ASTBuilder`
    - 识别控制结构: `ControlFlowStructureIdentifier`
    - 添加变量定义: `._add_variable_declarations()`
    - 递归地生成代码: `._build_block_statements()`
  - 添加全局变量定义: `._collect_and_generate_global_variables()`
  - 生成代码: `CSharpCodeGenerator.generate()`

