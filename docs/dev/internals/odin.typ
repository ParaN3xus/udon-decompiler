#import "/docs/book.typ": book-page
#import "@preview/shiroa:0.3.1": shiroa-sys-target
#import "../../_utils/common.typ": cross-link-heading

#show: book-page.with(title: "Odin Serializer")

为了摆脱对 Unity 环境的依赖, 我们参考官方实现重新编写了 Odin Serializer 的节点树 API, 然后在此基础上针对性地对 `UdonProgram` 和 `UdonVariableTable` 进行了更高层级 API 的实现.

