#import "/docs/book.typ": book-page
#import "@preview/shiroa:0.3.1": shiroa-sys-target

#show: book-page.with(title: "IR 和 IR 变换")

本节介绍函数 IR 的构建方式, `src/decompiler/transform/*` 中的变换框架, 以及默认 IR 变换管线中各 pass 的作用.

= 构建 IR

IR 的构建发生在数据流分析之后, 入口是 `build_ir_functions`.

对每个已经识别出的函数, 反编译器会遍历其 `FunctionCfg.block_ids`, 逐块读取原始指令, 并利用堆栈模拟阶段记录的 `instruction_states` 将指令翻译为机械化的 IR 语句.

在这个阶段, 常见指令会被翻译为如下 IR:
- `COPY` -> `IrAssignmentStatement`
- `EXTERN` -> `IrExpressionStatement` 或赋值语句
- 普通 `JUMP` -> `IrJump`
- returning call / tail call 形态的 `JUMP` -> `IrInternalCallExpression` 和 `IrReturn`
- `JUMP_IF_FALSE` -> `IrIf`
- 已识别 switch scaffold 的 `JUMP_INDIRECT` -> `IrSwitch`
- 返回语义的 `JUMP_INDIRECT` -> `IrLeave`

= IR 变换

IR 变换的入口是 `TransformPipeline::run`. 它分两段执行:
- 函数级 transforms: 依次对每个 `IrFunction` 执行
- 程序级 transforms: 在所有函数都完成函数级变换后执行

== Transform 框架

当前主要有三层接口:
- `ITransform`: 函数级 pass, 输入是一个 `IrFunction`
- `IProgramTransform`: 程序级 pass, 输入是整个函数数组
- `IBlockTransform` / `IStatementTransform`: 更细粒度的 block / statement 变换接口

为了让各个 pass 共享必要信息, 框架提供了两层 context:
- `ProgramTransformContext`: 整个变换阶段共享
- `TransformContext`: 单个函数 pass 运行时使用, 内部持有对 `ProgramTransformContext` 的可变借用

`ProgramTransformContext` 里目前最重要的内容包括:
- `class_name`: 最终输出类名
- `ir_class`: 程序级 transforms 构造出的类 IR
- `decompile_context`: 原始反编译上下文, 供各 pass 查询变量、原始符号、模块信息等
- `metadata`: 跨 pass 共享的小型状态表. 目前主要用于保存 `METADATA_SYNTHETIC_BLOCK_ADDR`, 供多个结构化 pass 分配 synthetic block address, 防止不同 pass 生成冲突的人工块地址.


== 默认 Transform 管线


默认管线由 `build_default_pipeline` 构造. 当前顺序为
- 函数级
  - `ControlFlowSimplification`: 折叠 jump chain 并清理空块
  - `ConstToLiteral`: 将常量变量引用改写为字面量
  - `TempVariableInline`: 内联最明显的临时变量赋值
  - `DetectExitPoints(false)`: 第一轮退出点检测, 为结构恢复做准备
  - `LoopDetection`: 识别 loop / switch body 并构造容器
  - `DetectExitPoints(true)`: 在容器成形后再做一轮退出点检测
  - `ConditionDetection`: 把典型 block 跳转形态改写为条件结构
  - `HighLevelLoopTransform`: 继续提升 loop container 的控制流结构
  - `HighLevelSwitchTransform`: 将低层 switch 容器提升为高层 switch 语句
  - `HighLevelLoopStatementTransform`: 将 loop container 提升为 `while` / `do-while`
  - `StructuredControlFlowCleanupTransform`: 清理残留 goto 风格结构
  - `CollectLabelUsage`: 重新标记仍需输出的块标签和出口标签
  - `CollectVariables`: 收集函数最终实际使用到的变量声明
- 程序级
  - `IrClassConstructionTransform`: 将所有函数组合为类级 IR
  - `PromoteGlobals`: 将共享变量提升到类级声明

