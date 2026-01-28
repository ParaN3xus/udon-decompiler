#import "/docs/book.typ": book-page, cross-link, heading-reference
#import "@preview/shiroa:0.3.1": shiroa-sys-target
#import "../../_utils/common.typ": cross-link-heading

#show: book-page.with(title: "Udon VM")


Udon VM 是一个简单的栈式虚拟机.

== 堆, 栈和寄存器

- 堆: 是一个 `IStrongBox[]`, 地址就是数组索引, 使用程序中的常量段初始化
- 栈: 一个 `u32` 栈
- PC 寄存器: 单位是字节


== 外部函数

Udon VM 的外部函数委托是 `UdonExternDelegate`, 具体定义为
```cs
delegate void UdonExternDelegate(IUdonHeap heap, Span<uint> parameterAddresses);
```
也即传入
- 堆用于获取参数和写入结果
- 一系列参数地址(在堆中的)用于获取参数

在此基础上封装了 `CachedUdonExternDelegate`, 具体定义为
```cs
class CachedUdonExternDelegate
{
  public readonly string externSignature;
  public readonly UdonExternDelegate externDelegate;
  public readonly int parameterCount;
}
```

`CachedUdonExternDelegate` 可以完全通过一个 `string` 获取, 也即 `externSignature`.

这个 `externSignature` 是 Udon Node 的方法签名, 相关生成代码在 #raw("UdonSharp.\u{200b}Compiler.\u{200b}Udon.\u{200b}CompilerUdonInterface") 中, 一些签名的例子如

#raw(
  block: true,
  lang: "cs",
  "UnityEngineGameObject.__SetActive__SystemBoolean__SystemVoid
VRCDynamicsVRCConstraintSource.__set_ParentPositionOffset__\u{200b}UnityEngineVector3
ExternVRCEconomyIProduct.__get_Name__SystemString
UnityEngineColor.__op_Addition__UnityEngineColor_UnityEngineColor__\u{200b}UnityEngineColor",
)


这些名字由两部分组成, 分别是 `ModuleName` 和 `FuncSignature`. 类(也即 `Module`)通过实现 `IUdonWrapperModule`, 将自己的 `ModuleName` 和所有 `FuncSignature` 及其对应的参数数量注册到 `UdonWrapper` 中, 供其使用完整的 `externSignature` 获取.

== 内部函数

除了#cross-link-heading("/dev/udon/udon-program.typ", [= 入口点表])[入口点表]一节中提到的入口点表外, Udon Sharp 在生成函数时候还做了其他的处理.

大多数(包括非公开的)函数有两个入口: 公开入口和内部入口. 公开入口用于外部调用, 从公开入口进入函数, 其执行结果是 Udon VM 停机. 从内部入口进入函数, 其执行结果是跳回到调用函数的 `JUMP` 之后.

两者的区别是, 公开入口比内部入口多了一句 `PUSH __const_SystemUInt32_0`, 这里 `__const_SystemUInt32_0` 的地址不固定, 但是其值永远是 `4294967295`, 也即 `0xFFFFFFFF`. 当这个值被写入 PC(也即 `JUMP` 到这个地址) 时, Udon VM 会停机.

函数的返回被编译为
```asm
PUSH, __intnl_returnJump_SystemUInt32_0
COPY
JUMP_INDIRECT, __intnl_returnJump_SystemUInt32_0
```

此处 `__intnl_returnJump_SystemUInt32_0` 的名字固定, 且堆地址总是是 `2`. 当通过公开入口进入时, 这里的 `__intnl_returnJump_SystemUInt32_0` 也就被写入了 `0xFFFFFFFF`, 最终使 Udon VM 停机. 而当内部入口进入时, 则是由调用者负责在调用前在堆中压入返回地址(也即 `JUMP` 指令的下一条指令的地址).

还有一部分函数没有公开入口.


== 执行过程
不断读取当前 PC 处的指令并执行, 直到停机或 PC 超出当前程序有效指令空间或 PC 为 `0xFFFFFFFF`. 不同指令的执行策略为
- `NOP`: PC 步进 4 字节
- `ANNOTATION`: PC 步进 8 字节
- `PUSH`: 把 `OPERAND` 作为立即数压栈, PC 步进 8 字节
- `POP`: 弹栈, 丢弃栈顶值, PC 步进 4 字节
- `JUMP`: 设置 PC 为立即数 `OPERAND`
- `JUMP_IF_FALSE`: 栈顶是堆地址, 弹栈, 读该地址对应的堆元素(`bool`)的值
  - 若为 `true`, PC 步进 8 字节
  - 若为 `false`, 设置 PC 为立即数 `OPERAND`
- `JUMP_INDIRECT`: 设置 PC 为 `OPERAND` 作为堆地址指向的 `u32` 值
- `EXTERN`: 调用外部函数. 尝试读取 `OPERAND` 作为堆地址指向的对象
  - 若为 `string`, 通过 `UdonWrapper` 获取该 `string` 对应的 `CachedUdonExternDelegate` (这是通常的情况)
  - 若为 `CachedUdonExternDelegate`, 也得到了 `CachedUdonExternDelegate`

  从栈中连续弹出 `CachedUdonExternDelegate.parameterCount` 个参数地址, 按与弹栈相反的顺序(也即最初的栈顶为最后一个地址)组装成 `Span<uint> parameterAddresses`, 并调用 `UdonExternDelegate`. PC 步进 8 字节

  在调用者(对于非静态函数)或返回值存在的情况下, 调用者和返回值分别作为第一个和最后一个参数传入.
- `COPY`: 从栈中先后弹出 `TARGET` 和 `SOURCE` 两个地址, 然后把堆中 `TARGET` 地址指向的值使用 `SOURCE` 地址指向的值覆盖. 所在 PC 步进 4 字节

从这里也可以看出, 栈中的值都是堆地址. 也即除了 `JUMP` 和 `JUMP_IF_FALSE` 之外的所有指令的 `OPERAND` 值都是堆地址.
