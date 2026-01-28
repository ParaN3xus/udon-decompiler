#import "/docs/book.typ": book-page
#import "@preview/shiroa:0.3.1": shiroa-sys-target
#import "../../_utils/common.typ": cross-link-heading

#show: book-page.with(title: "数据流分析")

本节主要介绍 `DataFlowAnalyzer` 和 `FunctionDataFlowAnalyzer` 类及其它一些相关的类的工作.

= 构建 CFG
== 识别入口点
入口点信息 `EntryPointInfo` 被定义为

```py
class EntryPointInfo:
    name: Optional[str]
    address: int
    call_jump_target: int
```

其中
- `name` 是函数名
- `call_jump_target` 是该函数从程序内部调用时, `JUMP` 指令的 `OPERAND`,
- `address` 是函数从外部调用时进入的公开入口, 若没有公开入口, 则使用 `call_jump_target` 替代

正如#cross-link-heading("/dev/udon/udon-program.typ", [= 入口点表])[入口点表]所述, 公开函数的入口已经标注在了 `UdonProgram.EntryPoints`.

一部分私有函数也有固定形式的公开入口 `PUSH __const_SystemUInt32_0`, 因此可以被简单地识别.

还有一部分私有函数(隐藏入口点)没有固定形式的公开入口, 我们通过多次迭代, 在已识别的函数中识别特殊形式的 `JUMP` 指令的方式识别它们.

=== 识别隐藏入口点 <sect:identify-hidden-entry-points>

基本思想是当 `JUMP` 时, 检查栈顶是否有值, 该值是否为 `JUMP` 指令的下一条指令的地址.

具体实现: 重复执行下面的流程, 直到没有出现新的入口点:

+ 按当前已知的入口点划分基本块
+ 在每个基本块中寻找调用函数的 `JUMP` 指令: `_find_call_targets`
  + 初始化 `BlockStackSimulator`, `state: StackFrame` 和 `HeapSimulator`
  + 模拟堆和栈的运行, 直到找到这样的 `JUMP` 指令:
    - 在该指令执行前, 栈顶有值
    - 该值恰好等于 `JUMP` 指令的下一条指令的地址
  + 将该 `JUMP` 指令的 `OPERAND` 记录到新的入口点中
+ 用新的入口点中更新已知的入口点

== 划分基本块

在#link(<sect:identify-hidden-entry-points>)[识别隐藏入口点]完成后, 基本块实际上已经完成划分. 因此本节讲述的是上述流程的一部分, 而不是一个独立的步骤.

基本上分为两步
- 找到块的开头
- 以块的开头所在的指令为分割点将指令顺序地分为基本块

其中块的开头包括
- `EntryPointInfo.address`
- `JUMP` 和 `JUMP_IF_FALSE` 指令的 `OPERAND` 和下一条指令的地址
- `JUMP_INDIRECT` 指令执行时, 其 `OPERAND` 作为堆地址指向的值

可以看出, 真正棘手的部分是 `JUMP_INDIRECT`. 实际上, 经过我的调研, 我发现 UdonSharp 编译器并不会轻易产生 `JUMP_INDIRECT`. 目前已知的 `JUMP_INDIRECT` 有两种:
- 内部函数的返回: 在基本块划分中, 它们可以被忽略, 因为这条 `JUMP_INDIRECT`
  - 要么让 UdonVM 停机: 此时并没有指示任何块的开头
  - 要么跳转到调用该函数的 `JUMP` 指令的下一条指令: 此时该开头已经在处理该 `JUMP` 指令时识别过
- 长的 `switch-case` 语句: 一些 switch expression 类型为 `int` 等的 `switch-case` 语句会被编译成从一个地址表中获取值, 然后 `JUMP_INDIRECT`. 这种类型的编译结果有明显可识别的模式(见 `BasicBlockIdentifier._get_switch_targets()`), 只需要匹配之然后把地址表中的所有值都识别为块的开头

可能还有更多类型的 `JUMP_INDIRECT`, 或者还有一些因为 `_get_switch_targets()` 的瑕疵而未能成功识别的 `switch-case` 类型的 `JUMP_INDIRECT`. 该问题在 #link("https://github.com/ParaN3xus/udon-decompiler/issues/4")[udon-decompiler/issues/4] 中被追踪.

== 构建边

这一步并不产生 `networkx.DiGraph`, 那是#link(<sect:build-function-cfg>)[构建函数 CFG]的任务. 这一步的任务只是正确地设置 `BasicBlock` 的 `predecessors` 和 `successors`, 以备接下来构建函数 CFG, 因此一些与函数内的 CFG 无关的边可以直接忽略.

具体的方法是考察每个 `BasicBlock` 的最后一条指令, 按 `OpCode` 分类处理
- `JUMP`:
  - 若为函数调用, 也即跳转目标是一个入口点: 建立本块到按指令地址顺序的下一块的边
  - 若不为函数调用: 建立本块到 `OPERAND` 起始的块的边
- `JUMP_IF_ELSE`: 建立本块到按指令地址顺序的下一块的边, 和本块到 `OPERAND` 起始的块的边
- `JUMP_INDIRECT`: 如上所述, 有两种
  - 内部函数的返回: 忽略
  - 长的 `switch-case` 语句: 建立本块的所有 `case` 的基本块的边
- 其它: 建立本块到按指令地址顺序的下一块的边

== 构建函数 CFG <sect:build-function-cfg>

从入口点所在的块开始 DFS 函数的所有块, 然后按 `BasicBlock.successors` 构建 `ControlFlowGraph` 对象内部的 `nx.DiGraph`.

= 函数数据流分析

本节尚未完成, 若要理解项目结构, 请自行查阅源代码.

== 栈模拟
== 识别变量
== 构建表达式
