#import "/docs/book.typ": book-page
#import "@preview/shiroa:0.3.1": shiroa-sys-target
#import "../../_utils/common.typ": cross-link-heading

#show: book-page.with(title: "数据流分析")

*这些内容已经过时, 仅供参考*


本节介绍 `DataFlowAnalyzer`/`FunctionDataFlowAnalyzer` 及其相关组件.

= 构建 CFG

`CFGBuilder` 的主要职责是:

- 基于字节码和入口点划分基本块
- 在构边时识别隐藏函数入口
- 为每个函数构建 `ControlFlowGraph`

== 入口点模型

入口点信息 `EntryPointInfo` 被定义为

```py
class EntryPointInfo:
    name: Optional[str]
    address: int
    call_jump_target: int
```

其中
- `name` 是函数名
- `call_jump_target` 是该函数从程序内部调用时 `JUMP` 指令的目标地址
- `address` 是函数的公开入口地址

正如#cross-link-heading("/dev/udon/udon-program.typ", [= 入口点表])[入口点表]所述, 公开函数的入口已经标注在了 `UdonProgram.EntryPoints`.

此外, `BytecodeParser` 会在反汇编后修正公开函数头: 若入口起始指令是 `PUSH __const_SystemUInt32_0` 且值为 `0xffffffff`, 则 `call_jump_target = address + 8`

== 划分基本块

基本思路分两步:
- 找到块的开头
- 以块的开头所在的指令为分割点将指令顺序地分为基本块

其中块的开头包括
- 已知入口点地址 (`EntryPointInfo.address` 与 `call_jump_target`)
- `JUMP` 和 `JUMP_IF_FALSE` 指令的 `OPERAND` 和下一条指令的地址
- 可识别的 `JUMP_INDIRECT` switch table 目标地址 (`BasicBlockIdentifier._get_switch_targets`)

`JUMP_INDIRECT` 里有两类被重点处理:
- 函数返回跳转 (`__intnl_returnJump_SystemUInt32_0`): 视为返回语义
- switch 地址表跳转: 识别后把所有 case target 作为块起点

== 构建边

这一步并不产生 `networkx.DiGraph`, 那是#link(<sect:build-function-cfg>)[构建函数 CFG]的任务. 这一步的任务只是正确地设置 `BasicBlock` 的 `predecessors` 和 `successors`, 以备接下来构建函数 CFG, 因此一些与函数内的 CFG 无关的边可以直接忽略.

具体的方法是考察每个 `BasicBlock` 的最后一条指令, 按 `OpCode` 分类处理
- `JUMP`:
  - 若识别为 returning-call, 弹出返回地址后建立 fallthrough 边
  - 若像函数调用但不返回, 将块标记为 `RETURN`
  - 否则是普通跳转边
- `JUMP_IF_FALSE`: 建立 false-target 与 fallthrough 两条边
- `JUMP_INDIRECT`:
  - 返回跳转: 标记 `RETURN`
  - switch 跳转: 建立到各 case target 的边
- 其它: 建立本块到按指令地址顺序的下一块的边

在这个阶段, `CFGBuilder` 还会发现隐藏入口点:
- 如果 `JUMP` 看起来像函数调用目标, 会将目标地址注册为新入口
- 新入口会加入队列继续分析, 直到不再产生新的入口

== 构建函数 CFG <sect:build-function-cfg>

从入口点所在的块开始 DFS 函数的所有块, 然后按 `BasicBlock.successors` 构建 `ControlFlowGraph` 对象内部的 `nx.DiGraph`.

== 识别函数名

公开函数名直接来自入口点表.

非公开函数的函数名可以函数的返回值变量符号名判断.  UdonSharp 中生成函数返回值变量的逻辑在 `UdonSharp.Compiler.CompilationContext.BuildMethodLayout` 中. 相关代码决定了返回值变量的符号名是

```
__{id1}___{id2}_{methodName}__ret
```

因此只需要在函数的指令中寻找对这种这种特殊名字的变量的写入即可. 目前, 反编译器只考虑了简单的 `COPY` 指令. 实际程序中还有一些 `EXTERN` 指令可以用于推测, 或许可以把这部分内容推迟到栈模拟之后, 相关工作在代码中有 `todo:` 记号.

作为回退策略, 反编译器会为函数生成一个临时函数名.

= 函数数据流分析

`FunctionDataFlowAnalyzer` 对每个函数执行:
- 栈模拟 (`StackSimulator`)
- 变量识别 (`VariableIdentifier`)
- IR 构建 (`IRBuilder`)

== 栈模拟

模拟结果会记录为每条指令执行前的 `StackFrame`, 后续变量识别与 IR 构建都会使用.

按拓扑遍历顺序遍历函数的所有基本块, 对于每个基本块, 按顺序模拟其每条指令的执行对栈产生的影响, 从而获得每一条指令运行前后的栈状态. 按 `OpCode` 不同, 具体的模拟逻辑如下
- `NOP`, `ANNOTATION`: 跳过
- `PUSH`: 压栈
- `POP`, `JUMP_IF_FALSE`: 弹栈
- `JUMP`: 若为函数调用, 弹栈或停机. 相当于模拟函数返回内部跳回原地址时的操作.
- `JUMP_INDIRECT`: 若为返回, 停机, 否则跳过. 这两种处理是不同的, 因为有些函数的末尾有两次连续的返回, 而作为返回语句的 `JUMP_INDIRECT` 指令在基本块划分时被忽略了, 这就导致一些基本块的末尾可能有两次连续的返回语句. 如果不立即停机, 模拟第二个返回语句时, 模拟器会因为尝试从空栈中弹出值而导致程序崩溃.
- `EXTERN`: 从堆中获取函数的 `externSignature`, 从栈中弹出对应数量的值
- `COPY`: 弹栈两次

== 识别变量

这一步的工作包括从符号表中识别变量和变量作用域, 然后记录函数的基本块中的变量读写, 最终达到识别变量的目的.

据观察, Udon 程序的变量在堆中位置是不变的, 而且一个萝卜一个坑, 不会出现一个地址被多个变量使用的情况, 这给我们带来了很大的方便.

=== 从符号表中识别变量

`UdonSharp.Compiler.Emit.ValueTable.GetUniqueValueName` 中有符号名的生成逻辑, 其中有一个对 `flags` 的 `switch-case` 决定了 `namePrefix`. 这让我们可以通过匹配符号名的开头得知其类型.

`flags` 的类型是 `Value.ValueFlags`, UdonSharp 源码中对这些 `ValueFlags` 给出了注释, 让我们能了解它们的用途以及可以被什么产生, 具体内容请查阅 UdonSharp 源码对 `enum ValueFlags` 的定义.

总之通过匹配符号名的开头, 我们能够得到变量的 `VariableScope`:
- `__const_`: 会在之后被消除, 此处作 `GLOBAL` 处理
- `__intnl_`: 会在之后被消除, 识别为 `TEMPORARY`
- `__gintnl_`: 大概率在之后被消除, 此处作 `GLOBAL` 处理
- `__lcl_`: 识别为 `LOCAL`
- `__this_`: 按名称转为 `this`/`this.transform`/`this.gameObject`, 作用域按 `GLOBAL` 处理
  - `__this_VRCUdonUdonBehaviour_{id}`: `this`
  - `__this_UnityEngineTransform_{id}`: `this.transform`
  - `__this_UnityEngineGameObject_{id}`: `this.gameObject`
- fallback: 识别为 `GLOBAL`

=== 记录变量读写

会记录 `PUSH`/`COPY`/`EXTERN` 导致的读写关系.
如果某地址未在符号表中出现, 会按回退策略创建临时变量 (`temp_{id}`).

== 构建原始 IR

`IRBuilder` 以基本块为单位构建 `IRBlockContainer`, 并把指令映射成 IR 语句:
- `COPY` -> `IRAssignmentStatement`
- `EXTERN` -> 外部调用/属性访问/构造/运算符表达式
- `JUMP` -> 内部调用或 `IRJump`
- `JUMP_IF_FALSE` -> `IRIf`
- `JUMP_INDIRECT` -> 返回或 `IRSwitch`

最后补齐隐式 fallthrough 或返回语句, 使每个块有明确终结行为.

= IR 变换管线 (`TransformPipeline`)

`DataFlowAnalyzer.analyze()` 在收集所有函数 IR 后, 会运行默认管线.

== 函数级 transforms

执行顺序如下:

+ `ControlFlowSimplification`
+ `ConstToLiteral`
+ `TempVariableInline`
+ `DetectExitPoints(can_introduce_exit_for_return=False)`
+ `LoopDetection` (`BlockILTransform`)
+ `DetectExitPoints(can_introduce_exit_for_return=True)`
+ `ConditionDetection` (`BlockILTransform`)
+ `HighLevelLoopTransform`
+ `HighLevelSwitchTransform`
+ `HighLevelLoopStatementTransform`
+ `StructuredControlFlowCleanupTransform`
+ `CollectLabelUsage`
+ `CollectVariables`

这些 pass 负责把低级跳转结构逐步转为高层 `while/do-while/switch` 或更简洁的条件分支.

== 程序级 transforms

- `IRClassConstructionTransform`: 组装 `IRClass`（类名/命名空间/函数列表）
- `PromoteGlobals`: 提升全局变量和跨函数共享变量为类级字段

此后, IR 会交给代码生成阶段输出伪 C\#.
