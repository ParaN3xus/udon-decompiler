#import "/docs/book.typ": book-page
#import "@preview/shiroa:0.3.1": shiroa-sys-target
#import "../../_utils/common.typ": cross-link-heading

#show: book-page.with(title: "数据流分析")


本节介绍 `run_analysis_pipeline` 函数中的内容.

= 变量识别

这一步的工作包括从符号表中识别变量和变量作用域.

据观察, 一般 Udon 程序的变量在堆中位置是不变的, 而且一个萝卜一个坑, 不会出现一个地址被多个变量使用的情况, 这给我们带来了很大的方便.

== 从符号表中识别变量

`UdonSharp.Compiler.Emit.ValueTable.GetUniqueValueName` 中有符号名的生成逻辑, 其中有一个对 `flags` 的 `switch-case` 决定了 `namePrefix`. 这让我们可以通过匹配符号名的开头得知其类型.

`flags` 的类型是 `Value.ValueFlags`, UdonSharp 源码中对这些 `ValueFlags` 给出了注释, 让我们能了解它们的用途以及可以被什么产生, 具体内容请查阅 UdonSharp 源码对 `enum ValueFlags` 的定义.

总之通过匹配符号名的开头, 我们能够得到变量的 `VariableScope`:
- `__const_`: 会在之后被消除, 此处作 `Global` 处理
- `__intnl_`: 会在之后被消除, 识别为 `Temporary`
- `__gintnl_`: 大概率在之后被消除, 此处作 `Global` 处理
- `__lcl_`: 识别为 `Local`
- `__this_`: 按名称转为 `this`/`this.transform`/`this.gameObject`, 作用域按 `Global` 处理
  - `__this_VRCUdonUdonBehaviour_{id}`: `this`
  - `__this_UnityEngineTransform_{id}`: `this.transform`
  - `__this_UnityEngineGameObject_{id}`: `this.gameObject`
- fallback: 识别为 `Global`


= 划分基本块

基本思路分两步:
- 找到块的开头
- 以块的开头所在的指令为分割点将指令顺序地分为基本块

其中块的开头包括
- 已知入口点地址 (`EntryPointInfo.address` 与 `call_jump_target`)
- `JUMP` 和 `JUMP_IF_FALSE` 指令的 `OPERAND` 和下一条指令的地址
- 可识别的 `JUMP_INDIRECT` switch table 目标地址 (`BasicBlockIdentifier._get_switch_targets`)

`JUMP_INDIRECT` 里有两类被重点处理:
- 函数返回跳转 (`__intnl_returnJump_SystemUInt32_0`): 视为返回语义
- switch 地址表跳转: 识别后把所有 case target 作为块起点. 此外, 还需要把该 switch 信息记录下来, 以供堆栈模拟阶段使用


= 堆栈模拟, 函数发现和 CFG 构建

Udon 没有函数调用指令, 因此我们必须利用堆栈模拟的结果, 通过一些启发式的方法来识别函数和函数调用. 这也是堆栈模拟, 函数发现和 CFG 构建这三个任务必须同时进行的原因.

基本上, 反编译器会按 BFS 顺序遍历所有块, 进行堆栈模拟. 队列的初始值就是程序的所有公开入口. 在堆栈模拟的过程中, 我们能发现新的函数和块的后继. 它们会被加入到队列中继续进行模拟.

== 堆栈模拟

每个块进行堆栈模拟的初始值是其前驱堆栈模拟的结果, 对于入口点所在的块, 堆的初始值是包含一个特殊的 `StackValue::HaltJump` 的栈.

- `NOP`, `ANNOTATION`: 忽略
- `PUSH`: 压栈
- `POP`: 弹栈
- `COPY`: 从栈中先后弹出 `TARGET` 和 `SOURCE` 两个地址, 然后把堆中 `TARGET` 地址指向的值设置为 `SOURCE` 地址指向的值覆盖
- `EXTERN`: 调用 `OPERAND` 所代表函数. 也即从栈中弹出对应参数个数个值. 若函数有返回值, 将对应堆地址设置为 `HeapValue::Unknown`
- `JUMP_IF_FALSE`: 建立 false-target 与 fallthrough 两条边
- `JUMP_INDIRECT`: 若 `OPERAND` 为 `HeapValue::HaltJump` 或其值超出程序地址, 则视为返回跳转. 若该块有 switch 信息, 则视为 switch 跳转. 否则为未识别跳转
  - 返回跳转: 标记该块为 `BasicBlockType::Return`
  - switch 跳转: 建立到各 case target 的边
- `JUMP`:
  - 若栈顶为下一条指令的地址, 则认为这是一个 returning call, 弹出返回地址后建立 fallthrough 边
  - 否则, 若跳转地址为已知的函数或跳转地址附近有类似函数头的结构, 则认为这是一个 trailing call, 注册新的函数, 标记该块为 `BasicBlockType::Return`
  - 否则建立普通跳转边

== 函数发现

如上所述, 我们主要通过模拟 `JUMP` 指令来发现新的函数.

需要注意的是, 在识别了一个新的函数后, 我们需要重新模拟与这个函数相关的块, 包括
- 函数入口点所在的块及其后代
- JUMP 到这个地址的块. 不需要模拟后代的原因是 returning call 不可能在函数被识别前被忽略, 而 trailing call 没有后继

== CFG 构建 <sect:build-function-cfg>

边关系已经在堆栈模拟阶段构建完成. 之后我们还需要对每个函数构建 CFG: 从入口点所在的块开始 DFS 函数的所有块, 然后按全局 CFG 中的边关系构建函数 CFG 中的边关系.

== 识别函数名

公开函数名直接来自入口点表.

非公开函数的函数名可以函数的返回值变量符号名判断.  UdonSharp 中生成函数返回值变量的逻辑在 `UdonSharp.Compiler.CompilationContext.BuildMethodLayout` 中. 相关代码决定了返回值变量的符号名是

```
__{id1}___{id2}_{methodName}__ret
```

因此只需要在函数的指令中寻找对这种这种特殊名字的变量的写入即可. 目前, 反编译器只考虑了简单的 `COPY` 指令. 实际程序中还有一些 `EXTERN` 指令可以用于推测, 或许可以把这部分内容推迟到栈模拟之后, 相关工作在代码中有 `todo:` 记号.

作为回退策略, 反编译器会为函数生成一个临时函数名.

