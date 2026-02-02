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
- 内部函数的返回: #cross-link-heading("/dev/udon/udon-vm.typ", [= 内部函数])[Udon VM - 内部函数]一节中指出了函数返回语句的特征, 使得我们可以很容易地识别它们. 在基本块划分中, 它们可以被忽略, 因为这条 `JUMP_INDIRECT`
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

== 识别函数名

公开函数的函数名可以在 `UdonProgram` 的入口点表中找到.

非公开函数的函数名可以函数的返回值变量符号名判断.  UdonSharp 中生成函数返回值变量的逻辑在 `UdonSharp.Compiler.CompilationContext.BuildMethodLayout` 中. 相关代码决定了返回值变量的符号名是

```
__{id1}___{id2}_{methodName}__ret
```

因此只需要在函数的指令中寻找对这种这种特殊名字的变量的写入即可. 目前, 反编译器只考虑了简单的 `COPY` 指令. 实际程序中还有一些 `EXTERN` 指令可以用于推测, 或许可以把这部分内容推迟到栈模拟之后, 相关工作在代码中有 `todo:` 记号.

作为回退策略, 反编译器会为函数生成一个临时函数名.

= 函数数据流分析

== 栈模拟

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
- `__this_`: 是对 `this` 的成员的引用, 并不会需要变量声明, 此处作 `GLOBAL` 处理.
- fallback: 识别为 `GLOBAL`

除此之外, 由于对 `this`(是 `MonoBehaviour`) 的引用一共就那几种, 我们可以枚举并按符号名直接给出特殊的变量名
- `__this_VRCUdonUdonBehaviour_{id}`: `this`
- `__this_UnityEngineTransform_{id}`: `this.transform`
- `__this_UnityEngineGameObject_{id}`: `this.gameObject`

=== 记录变量读写

遍历函数的基本块和块内的指令, 记录 `PUSH`, `COPY`, `EXTERN` 对堆地址(也即变量)的读写. 此处我们忽略了 `JUMP_INDIRECT`, 因为函数返回和 `switch-case` 的跳转步骤都不构成对实际变量的读写.

在本步骤中, 如果出现了对未在上一步中识别出的变量的读写, 这些变量会命中回退策略, 并获得一个临时变量名.

== 构建表达式

遍历函数内的所有指令, 结合栈模拟的结果, 不同指令可以构建出下列表达式
- `COPY`: `ASSIGNMENT`
- `EXTERN`: `PROPERTY_ACCESS`, `CONSTRUCTOR`, `OPERATOR`, `EXTERNAL_CALL`
- `JUMP`: `INTERNAL_CALL`

在构建这些表达式时, 会将栈中的值, 也即堆地址同时构建为 `LITERAL` 或 `VARIABLE` 表达式.
