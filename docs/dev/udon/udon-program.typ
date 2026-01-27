#import "/docs/book.typ": book-page, heading-reference
#import "@preview/shiroa:0.3.1": shiroa-sys-target

#show: book-page.with(title: "Udon Program")

Udon Program 是 Udon Script 的编译产物, 每个 Udon Program 只代表一个类.

= 资产
使用 AssetRipper 解包地图后, 能得到大量 AssetRipper 无法正确解析的 MonoBehaviour 资产文件. 其中一些 MonoBehaviour 资产包含一个很长的 `serializedProgramCompressedBytes`. 这代表这个资产是一个 Udon Script 的编译产物.

`serializedProgramCompressedBytes` 是一个十六进制字符串, 是 GZip 压缩后的 Udon Program 序列化结果.

= Udon Program 的反序列化
`serializedProgramCompressedBytes` 经过 GZip 解压后得到的二进制文件是 `UdonProgram` 实例序列化后的结果.

这个序列化过程使用的是一个 VRChat 修改的 `OdinSerializer`. 所以我们可以直接用这个序列化器对应的反序列化器进行反序列化. 一些关键代码如下

```cs
using System.IO;
using VRC.Udon.Common;
using VRC.Udon.Serialization.OdinSerializer;

using var memoryStream = new MemoryStream(fileData);
var context = new DeserializationContext();
var reader = new BinaryDataReader(memoryStream, context);
UdonProgram program =
  VRC.Udon.Serialization.OdinSerializer.SerializationUtility
    .DeserializeValue<UdonProgram>(reader);
```

= `UdonProgram` 类

`UdonProgram` 类中几乎有我们需要的一切. 下面是一个简化的类定义
```cs
public class UdonProgram : IUdonProgram
{
  public string InstructionSetIdentifier { get; }
  public int InstructionSetVersion { get; }
  public byte[] ByteCode { get; }
  public IUdonHeap Heap { get; }
  public IUdonSymbolTable EntryPoints { get; }
  public IUdonSymbolTable SymbolTable { get; }
  public IUdonSyncMetadataTable SyncMetadataTable { get; }
  public int UpdateOrder { get; }
}
```

我们比较关心 `ByteCode`, `Heap`, `EntryPoints`, `SymbolTable` 这几个字段.

== Udon 字节码和指令集

是一系列大端序 `u32` 组成的指令的序列.

指令格式为 `OPCODE[OPERAND]`, 两部分各 4 字节, `OPERAND` 是一个大端序 `u32`.

`OPCODE` 包括无参数的 `NOP`, `POP`, `COPY` 和有一个参数的 `PUSH`, `JUMP_IF_FALSE`, `JUMP`, `EXTERN`, `ANNOTATION`, `JUMP_INDIRECT`.

各 `OPCODE` 对应的值为

```python
class OpCode(IntEnum):
    NOP = 0
    PUSH = 1
    POP = 2
    JUMP_IF_FALSE = 4
    JUMP = 5
    EXTERN = 6
    ANNOTATION = 7
    JUMP_INDIRECT = 8
    COPY = 9
```

各 `OPCODE` 和 `OPERAND` 含义如下：

- `NOP`: 空指令
- `PUSH I`: 将立即数 `I` 压栈
- `POP`: 从栈中弹出一个值并丢弃
- `COPY`: 复制堆中的值
- `JUMP_IF_FALSE ADDR`: 条件跳转到 `ADDR`
- `JUMP ADDR`: 无条件跳转到 `ADDR`
- `EXTERN F`: 调用外部函数, `F` 是堆中的函数签名 `string` 或者函数委托 `UdonExternDelegate` 的地址
- `ANNOTATION`: 注解, 执行时跳过
- `JUMP_INDIRECT IADDR`: 间接跳转到 `IADDR` 作为堆地址指向的值


== 堆

用于存储 Udon VM 执行该 Udon Program 时堆的初始值, 相当于常量段.

简化的类定义如下
```cs
[Serializable]
public sealed class UdonHeap : IUdonHeap, ISerializable
{
  [NonSerialized]
  private readonly IStrongBox[] _heap;

  public void GetObjectData(
    SerializationInfo info, StreamingContext context
  )
  {
    List<ValueTuple<uint, IStrongBox, Type>> list =
      new List<ValueTuple<uint, IStrongBox, Type>>();
    this.DumpHeapObjects(list);
    info.AddValue("HeapCapacity", Math.Max(0, this._heap.Length));
    info.AddValue("HeapDump", list);
  }

  public void DumpHeapObjects(
    List<ValueTuple<uint, IStrongBox, Type>> destination
  )
  {
    uint num = 0;
    while (num < this._heap.Length)
    {
      IStrongBox strongBox = this._heap[num];
      if (strongBox != null)
      {
        destination.Add(new ValueTuple<uint, IStrongBox, Type>(
          num,
          strongBox,
          strongBox.GetType().GenericTypeArguments[0]
        ));
      }
      num += 1;
    }
  }
}
```

我们感兴趣的就是其中的 `HeapDump`, 这是一个 `(Addr, Value, Type)` 三元组的列表.

== 入口点表 <sect-entry-points>

实际上是*公开*函数表.

简化的类定义如下
```cs
[Serializable]
public sealed class UdonSymbolTable : IUdonSymbolTable, ISerializable
{
  private readonly ImmutableArray<string> _exportedSymbols;
  private readonly ImmutableDictionary<string, IUdonSymbol> _nameToSymbol;

  void ISerializable.GetObjectData(
    SerializationInfo info, StreamingContext context
  )
  {
    info.AddValue(
      "Symbols",
      this._nameToSymbol.Values.ToList<IUdonSymbol>()
    );
    info.AddValue(
      "ExportedSymbols",
      this._exportedSymbols.ToList<string>()
    );
  }
}

[Serializable]
public sealed class UdonSymbol : IUdonSymbol, ISerializable
{
  public string Name { get; }
  public Type Type { get; }
  public uint Address { get; }

  void ISerializable.GetObjectData(
    SerializationInfo info, StreamingContext context
  )
  {
    info.AddValue("Name", this.Name);
    info.AddValue("Type", this.Type);
    info.AddValue("Address", this.Address);
  }
}
```

这里每个 `UdonSymbol` 里的
- `Name` 是函数名
- `Address` 是该函数的首条指令在 `UdonProgram.ByteCode` 中的索引
- `Type` 对于入口点来说, 无意义

这给我们带来了很多方便.

== 符号表

类定义和入口点表相同, 其中每个 `UdonSymbol` 里的
- `Name` 是符号名
- `Address` 是该符号在堆中的地址
- `Type` 是符号类型
