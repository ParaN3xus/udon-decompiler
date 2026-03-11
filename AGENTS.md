# AGENTS.md

## 1. 项目定位

本仓库是 **Udon Decompiler 的 Rust 重写版**，目标不仅是功能迁移，还包括：
- 更清晰的分层（解析/IR/变换/代码生成）；
- 更稳定的可测试性（case + snapshot）；
- 为后续优化留出可维护扩展点。

Python 旧实现位于 `local/udon-decompiler`，作为行为对照与回归参考。

## 2. 协作优先级

当你（Agent）修改代码时，按以下顺序决策：
1. 正确性优先：先保证字节码语义、控制流恢复、extern/heap 语义不回归。
2. 可验证性优先：新增/修复行为时，优先补最小 case（`tests/cases/**.md`）和 snapshot。
3. 性能最后做：除非用户明确要求，不先做“猜测式优化”。

## 3. 当前代码分层（必须遵守）

- `src/odin/*`：UdonProgram 二进制文档模型解析/序列化/编辑。
- `src/udon_asm/*`：ASM 文本 <-> 字节码/Program 的编解码与指令/字面量处理。
- `src/decompiler/*`：反编译主流程。
  - `context.rs`：输入程序上下文加载与索引；
  - `pipeline.rs`：主流水线编排；
  - `cfg.rs`、`basic_block.rs`：控制流与栈模拟；
  - `ir/*`：IR 结构与构建；
  - `transform/*`：IR/程序变换管线；
  - `codegen.rs`：C# 代码生成。
- `src/main.rs`：CLI（`dc`/`dasm`/`asm`）。

不要把跨层逻辑硬塞到 CLI 或 util；尽量在对应层闭环。

## 4. 关键运行前置条件

- 反编译依赖 `UdonModuleInfo.json`（默认从当前工作目录读取，或通过 `--module-info` 指定）。
- 当前仓库未内置该文件；缺失时 `cfg/stack simulation` 会失败。
- 测试 `cargo test` 目前会因该文件缺失失败（`tests/e2e.rs`）。

如果你新增测试或命令说明，请明确是否依赖该文件，以及路径假设。

## 5. 建议工作流

1. 先复现：优先用最小输入（`tests/cases/basic`）确认问题。
2. 再定位：在 `decompiler::pipeline` 的阶段边界打点（变量识别、CFG、IR、transform、codegen）。
3. 最后回归：
   - 受影响 case 的 snapshot；
   - 至少一个 real_world case（如 `tests/cases/real_world/QvPen/*`）抽查。

## 6. 测试与命令

- 查看 CLI：`cargo run -- --help`
- 反编译：`cargo run -- dc <input.b64> --module-info <path/to/UdonModuleInfo.json>`
- 反汇编：`cargo run -- dasm <input.b64>`
- 回组装：`cargo run -- asm <input.asm> --template <original.b64>`
- 运行测试：`cargo test`

说明：`asm` 子命令的 `--template` 仅在 `asm` 模式有效。

## 7. 变更约束

- 变更尽量小步提交，避免一次性重构多个层级。
- 新增 public API 时更新 `src/decompiler/mod.rs` 或相关 `mod.rs` 的导出。
- 遇到不确定语义时，优先对齐旧版 Python 行为并在注释/文档中写明差异。
- 未经明确需求，不重命名大量符号、不批量格式化无关文件。

## 8. 文档更新要求

出现以下情况时，需要同步更新 `CONTEXT.md`：
- 新增 pipeline 阶段；
- 变更 transform 默认顺序；
- 修改测试数据组织方式；
- 引入新的外部前置文件（类似 `UdonModuleInfo.json`）。
