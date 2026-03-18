#import "/docs/book.typ": book-page
#import "@preview/shiroa:0.3.1": shiroa-sys-target

#show: book-page.with(title: "代码生成")

这部分内容比较直白, 基本上就是把前面构建的 IR 按原意翻译成 C\# 伪代码, 然后使用 clang-format 格式化. 具体实现请参阅源代码.
