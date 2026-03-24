#import "/docs/book.typ": book-page
#import "@preview/shiroa:0.3.1": shiroa-sys-target

#show: book-page.with(title: "概览")


本节介绍本项目概况, 代码仓库的文件结构和当前工作流程, 以便读者快速了解项目.

= 项目概况

本项目主要包括反编译器本体和一些与反编译器本体配合使用的 Unity Editor Script.

关于反编译器本体, 它当前是 Rust 实现. 反编译器负责解析 Udon Program、构建 CFG 和 IR、执行结构化 transforms, 并最终生成伪 C\# 代码.

= 代码仓库
```shell-unix-generic
$ git ls-files --exclude-standard | tree --fromfile -L 2
.
├── AGENTS.md
├── biome.json
├── Cargo.lock
├── Cargo.toml
├── docs                          # 你正在阅读的文档的源码
│   ├── about
│   ├── book.typ
│   ├── dev
│   ├── for-llm
│   ├── introduction.typ
│   ├── user
│   ├── user-en
│   └── _utils
├── LICENSE
├── package.json
├── README.md
├── src                           # 反编译器主体程序源码
│   ├── bin
│   ├── decompiler                # 反编译器模块
│   ├── lib.rs
│   ├── logging.rs
│   ├── main.rs
│   ├── odin                      # Odin 序列化器模块
│   ├── str_constants.rs
│   ├── udon_asm                  # 反汇编/汇编器模块
│   └── util
├── tests                         # e2e 测试
│   ├── cases                     # e2e 测试所用样例
│   ├── e2e.rs
│   └── snapshots
├── tools
│   ├── bump-version.ts
│   ├── case-md.ts
│   ├── Editor                    # 编辑器脚本
│   └── UdonProgramDumper         # UdonProgramDumper C# 项目
├── tsconfig.json
├── UdonDecompiler.slnx
└── yarn.lock

20 directories, 19 files
```

= 工作流程
== 反编译前

- 提取 `UdonModuleInfo.json`: 获取所有可能的 `externSignature` 对应的函数信息, 输出为 `.json` 供反编译器使用
- 读取 `serializedProgramCompressedBytes`: 读取 `.hex` 格式的 `serializedProgramCompressedBytes` 输入, 或从 Unity `.asset` 中提取该字段, 解压后解析为 `UdonProgramBinary`

== 反编译

这部分内容的入口在 `DecompileContext::from_program` 和 `run_decompile_pipeline` 函数中. 按调用顺序可分为如下步骤:

- 反汇编. 解析入口点, 符号, 堆信息
- 数据流分析
  - 变量识别: `VariableTable::identify_from_heap`
  - 基本块识别: `BasicBlockCollection::identify_from_context`
  - 堆栈模拟, 函数发现和 CFG 构建: `build_cfgs_and_discover_entries`
    - `simulate_and_discover`
    - 构建各函数的 CFG: `build_function_cfgs`
- 构建函数 IR: `build_ir_functions`
- 执行 IR 变换管线: `TransformPipeline`
  - 函数级 transforms: 控制流化简, 常量转字面量, 表达式内联, 退出点检测, 循环/条件结构识别, 高层循环变换, switch-case 识别, 控制结构清理, 收集块标签, 收集变量
  - 程序级 transforms: 构建类 IR, 提升全局变量
- 代码生成: `generate_csharp`
  - 从 IR 生成伪 C\# 代码
  - 运行 `clang-format` 进行格式化
