#import "/docs/book.typ": book-page
#import "@preview/shiroa:0.3.1": shiroa-sys-target
#import "../../_utils/common.typ": cross-link-heading

#show: book-page.with(title: "数据流分析")

本节尚未完成, 若要理解项目结构, 请自行查阅源代码.

= 构建 CFG
== 识别入口点
正如#cross-link-heading("/dev/udon/udon-program.typ", [= 入口点表])[入口点表]所述, 公开函数的入口已经标注在了 `UdonProgram.EntryPoints`.

== 划分基本块

== 构建边

== 构建每个函数的 CFG

= 函数数据流分析

== 栈模拟
== 识别变量
== 构建表达式
