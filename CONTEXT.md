# CONTEXT.md

## 项目快照

- 名称：`udon-decompiler`
- 语言：Rust（edition 2024）
- 形态：库 + CLI
- 核心目标：把 Udon `.b64` 程序恢复为可读 C#，并提供 `b64 <-> asm` 的可逆处理能力。

## 一句话理解架构

`Odin 二进制模型` -> `Udon 字节码/符号/堆抽取` -> `CFG + 栈模拟` -> `IR` -> `Transform Pipeline` -> `C# 代码生成`

## 仓库结构地图

- `src/main.rs`
  - CLI 入口，子命令：`dc`、`dasm`、`asm`。
- `src/lib.rs`
  - 暴露主要模块：`decompiler`、`odin`、`udon_asm` 等。
- `src/odin/*`
  - 底层二进制文档树读写（`UdonProgramBinary` 是核心入口）。
- `src/udon_asm/*`
  - 指令 opcode/operand 定义、ASM 解析、反汇编、回组装、heap literal 编解码。
- `src/decompiler/*`
  - 反编译主域。
  - `context.rs`：从程序提取 bytecode/heap/symbol/entry，构造 `DecompileContext`。
  - `pipeline.rs`：串联各阶段并产出 `generated_code`。
  - `cfg.rs`、`basic_block.rs`：基本块、函数入口发现、CFG 和栈模拟。
  - `ir/*`：IR 节点、控制流图、dominance、IR builder。
  - `transform/*`：默认 IR 变换链。
  - `codegen.rs`：IR -> C# 文本。
- `tests/e2e.rs`
  - 读取 `tests/cases/**/*.md` 中的 ```b64``` 代码块做 smoke + snapshot。
- `tests/cases/*`
  - 按 `basic` / `control_flow` / `real_world` 组织用例。
- `tests/snapshots/*`
  - `insta` 快照输出。

## 当前默认反编译流水线（代码事实）

`run_decompile_pipeline` 中的主要阶段：
1. 从 heap/symbol 识别变量（`VariableTable`）。
2. 识别基本块（`BasicBlockCollection`）。
3. 发现入口并构建 CFG + 栈模拟。
4. CFG 降级生成 IR functions。
5. 执行 transform pipeline。
6. 生成 C#。

默认 transform 顺序（IL transforms）：
1. `ControlFlowSimplification`
2. `ConstToLiteral`
3. `TempVariableInline`
4. `DetectExitPoints(false)`
5. `LoopDetection`
6. `DetectExitPoints(true)`
7. `ConditionDetection`
8. `HighLevelLoopTransform`
9. `HighLevelSwitchTransform`
10. `HighLevelLoopStatementTransform`
11. `StructuredControlFlowCleanupTransform`
12. `CollectLabelUsage`
13. `CollectVariables`

Program transforms：
1. `IrClassConstructionTransform`
2. `PromoteGlobals`

## CLI 行为摘要

已验证 `cargo run -- --help` 输出：
- `dc`：`b64 -> C#`
- `dasm`：`b64 -> asm`
- `asm`：`asm -> b64`

全局参数：
- `--log-level <LEVEL>`（默认 `info`）
- `--module-info <PATH>`（覆盖默认 `UdonModuleInfo.json`）

注意：`--template` 只允许在 `asm` 子命令下使用。

## 测试现状（2026-03-09 实测）

- 执行：`cargo test -q`
- 结果：`tests/e2e.rs` 中 2 个测试失败。
- 失败根因：工作目录缺少 `UdonModuleInfo.json`，导致 CFG/栈模拟阶段无法加载 extern 元数据。

这不是随机失败；是明确的环境前置缺失。

## 与 Python 版本的关系

- Rust 版是主线实现。
- Python 版位于 `local/udon-decompiler`，适合用来：
  - 对齐语义（尤其 extern 调用和控制流边界）；
  - 构造最小回归 case；
  - 对比输出差异并判断是 bug 还是改进。

## 建议的开发切入点

1. 想改反编译语义：从 `src/decompiler/pipeline.rs` 对应阶段进入。
2. 想改控制流恢复：看 `src/decompiler/cfg.rs` + `src/decompiler/transform/passes/*`。
3. 想改文本汇编格式：看 `src/udon_asm/parse.rs` 与 `src/udon_asm/disassemble.rs`。
4. 想补回归：在 `tests/cases/**.md` 添加 `b64` 片段并更新 snapshot。
