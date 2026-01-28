#import "/docs/book.typ": book-page, cross-link, heading-reference
#import "@preview/shiroa:0.3.1": shiroa-sys-target
#import "/docs/_utils/common.typ": cross-link-heading

#show: book-page.with(title: "Udon Variable Table")

这部分内容与反编译关系不大, 但是对理解甚至修改地图有用.

= 资产
使用 AssetRipper 解包地图后, 在场景或预制件中能找到一些 `MonoBehaviour`, 这些 `MonoBehaviour` 带有 `serializedPublicVariablesBytesString` 字段. 这代表这是一个对象绑定的脚本的公共字段的初始值.

`serializedPublicVariablesBytesString` 是一个 base64 编码字符串, 是 `UdonVariableTable` 的序列化结果. 该序列化过程同样使用了#cross-link-heading("/dev/udon/udon-program.typ", [= Udon Program 的反序列化])[Udon Program 的反序列化]一节中提到的 `OdinSerializer`, 反序列化的方式也一样, 此处不再赘述.

= `UdonVariableTable` 类

基本上就是一个 `Dictionary<string, IUdonVariable>`. 重要接口如下
```csharp
public IReadOnlyCollection<string> VariableSymbols;
public bool TryAddVariable(IUdonVariable variable);
public bool RemoveVariable(string symbolName);
public bool TrySetVariableValue(string symbolName, object value);
public bool TryGetVariableType(string symbolName, out Type type);
public bool TryGetVariableValue(string symbolName, out object value);
```

== `IUdonVariable`

```csharp
public interface IUdonVariable
{
    string SymbolName { get; }
    object Value { get; set; }
    Type DeclaredType { get; }
}
```

= 修改 `PublicVariables`

需要注意的是, `UdonVariableTable` 实例中可能含有一些引用, 因此在不完整的环境中将其反序列化后再次序列化会破坏其内容, 导致相关脚本无法正常运行.

因此, 在修改时要么使用更基础的 `DataNode` 相关 API, 要么保证环境完整, 并正确设置 `publicVariablesUnityEngineObjects`.
