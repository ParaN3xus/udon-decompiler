#import "/docs/book.typ": book-page
#import "@preview/shiroa:0.3.1": shiroa-sys-target
#import "../../_utils/common.typ": cross-link-heading

#show: book-page.with(title: "提取 UdonModuleInfo.json")

#cross-link-heading("/dev/udon/udon.typ", [== 执行过程])[Udon VM - 执行过程]和 #cross-link-heading("/dev/udon/udon.typ", [== 外部函数])[Udon VM - 外部函数]中解释了 Udon VM 如何调用外部函数. 为了正常识别外部函数调用, 并重建更高级的语义信息, 本反编译器也需要知道每个 `externSignature` 所对应函数的参数数量, 原始名字, 具体类型(`method`, `ctor`, `op`, `field`), 是否有返回值, 是否为静态函数等信息.

这一提取工作通过编辑器脚本 `UdonModuleInfoExtractor.cs` 完成, 其大致运行逻辑如下
- 使用 `UdonVM` 所使用的 API 获取所有实现了 `IUdonWrapperModule` 的类
- 对于每一个类, 读取它们的 `Name`(`externSignature` 中的 `ModuleName`) 和 `_parameterCounts`(`Dictionary<string, int>`) 字段. 从而获得每个 `externSignature` 对应的参数数量
- 使用 `NodeRegistries` 相关 API 获取每个 `externSignature` 对应的 `UdonNodeDefinition`
- 对于每个类中的每个函数, 使用其函数签名和 `UdonNodeDefinition` 中的参数列表信息推断我们需要的所有信息, 具体而言
  - 推断类的原始名字: 使用该类中任一函数的 `UdonNodeDefinition.type` 即可获得该类的 `Type`
  - 推断类型: 根据函数签名的开头和参数数量的不同
    - `__op_` 开头的, 是 `op` 类型
    - `__ctor__` 开头的, 是 `ctor` 类型
    - `__get_` 或 `__set_` 开头的且参数数量不小于 1, 不大于 2 的, 是 `field` 类型(签名符合特征但是参数数量不正确的那部分函数是数组成员访问. 截至目前, 这种类型的函数还没有被正确反编译, 相关工作在 todo 列表中)
    - 其它的是 `method` 类型
  - 推断原始名字
    - 对于 `field` 类型: 将函数签名去掉开头的 `__get_` 或 `__set_` 即为原始名字
    - 对于 `method` 类型: 将对应 `UdonNodeDefinition.name` 用空格 `rsplit`, 最后一段即为原始名字
  - 推断是否为静态函数: 检查 `UdonNodeDefinition.parameters.First()` 的 `name` 是否为 `instance` 且类型是否与该类相同
  - 推断是否有返回值: 检查 `UdonNodeDefinition.parameters.Last()` 的 `parameterType` 是否为 `UdonNodeParameter.ParameterType.OUT`
- 将所有信息输出到 `UdonModuleInfo.json` 中
