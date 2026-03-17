#import "/docs/book.typ": book-page
#import "@preview/shiroa:0.3.1": shiroa-sys-target

#show: book-page.with(title: "概览")

*这些内容已经过时, 仅供参考*


本节介绍本项目概况, 代码仓库的文件结构和当前工作流程, 以便读者快速了解项目.

= 项目概况

本项目主要包括反编译器本体和一些与反编译器本体配合使用的 Unity Editor Script.

关于反编译器本体, 它当前是 Rust 实现. 反编译器负责解析 Udon Program、构建 CFG 和 IR、执行结构化 transforms, 并最终生成伪 C\# 代码.

= 代码仓库
```shell-unix-generic
$ tree -L 2 --gitignore
.
├── docs                                      # 你正在阅读的文档
│   ├── about
│   ├── book.typ
│   ├── dev
│   ├── for-llm
│   ├── introduction.typ
│   ├── user
│   ├── user-en
│   └── _utils
├── Editor                                    # 编辑器脚本
│   ├── UdonModuleInfoExtractor.cs
│   ├── UdonSharpSourceTextCompilerCLI.cs
│   ├── UdonSharpSourceTextCompiler.cs
│   └── UdonSharpSourceTextCompilerGUI.cs
├── LICENSE
├── package.json
├── README.md
├── src                                       # Rust 反编译器源代码
├── tests                                     # Markdown case + snapshot 测试
└── tsconfig.json
```

= 工作流程
== 反编译前

- 提取 `UdonModuleInfo.json`: 获取所有可能的 `externSignature` 对应的函数信息, 输出为 `.json` 供反编译器使用
- 读取 `serializedProgramCompressedBytes`: 当前反编译器直接读取压缩后的 `.hex` 输入, 或从 Unity `.asset` 中提取该字段, 解压后解析为 `UdonProgram`

== 反编译

这部分内容的入口在 `decompile_program_to_source` 函数中. 按调用顺序可分为如下步骤:

- 反汇编: `BytecodeParser`
- 数据流分析: `DataFlowAnalyzer`
  - 构建 CFG: `CFGBuilder`
    - 识别基本块: `BasicBlockIdentifier`
    - 构建控制流边并发现隐藏函数入口: `._build_edges()`
    - 构建每个函数的 CFG 并识别函数名: `._build_function_cfgs()`
  - 函数级分析: `FunctionDataFlowAnalyzer`
    - 栈模拟: `StackSimulator`
    - 识别变量: `VariableIdentifier`
    - 构建原始 IR: `IRBuilder`
  - IR 变换: `TransformPipeline`
    - 函数级 transforms: 控制流化简、循环/条件识别、`while/do-while/switch` 抬升、变量收集等
    - 程序级 transforms: `IRClassConstructionTransform` 和 `PromoteGlobals`
- 代码生成: `ProgramCodeGenerator`
  - `CSharpCodeGenerator.generate()`: 从 IR 生成伪 C\# 代码
  - 运行 `clang-format` 进行格式化
