#import "/docs/book.typ": book-page
#import "/docs/_utils/common.typ": *

#show: book-page.with(title: "简介")

#udon-decompiler (下称"本项目")是一个针对 #link("https://creators.vrchat.com/worlds/udon/")[Udon] 的#link("https://en.wikipedia.org/wiki/Decompiler")[反编译器]\(下称"本反编译器")及相关工具的合集. 它可以将 #udon 程序的编译结果 --- Udon Program 反编译为 C\# 形式的伪代码.

= 许可证

本项目的源码和文档内容在 #link("https://www.gnu.org/licenses/agpl-3.0.html")[AGPL 3.0] 许可证下发布.
