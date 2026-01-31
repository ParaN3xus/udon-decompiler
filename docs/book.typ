
#import "@preview/shiroa:0.3.1": *

#show: book

#book-meta(
  title: "Udon Decompiler",
  description: "Udon Decompiler 文档",
  repository: "https://github.com/ParaN3xus/udon-decompiler/",
  repository-edit: "https://github.com/ParaN3xus/udon-decompiler/edit/main/docs/{path}",
  authors: ("ParaN3xus",),
  language: "zh",
  summary: [
    // begin of summary
    #prefix-chapter("introduction.typ")[简介]
    = 用户指南
    - #chapter("user/installation.typ")[安装]
    - #chapter("user/usage.typ")[使用]
    - #chapter("user/reporting-issues.typ")[汇报错误]
    = User Guide (EN)
    - #chapter("user-en/installation.typ")[Installation]
    - #chapter("user-en/usage.typ")[Usage]
    - #chapter("user-en/reporting-issues.typ")[Reporting Issues]
    = 开发指南
    - #chapter("dev/status.typ")[项目现状]
    - #chapter("dev/udon/main.typ")[Udon 和 UdonSharp 相关知识]
      - #chapter("dev/udon/udon-program.typ")[Udon Program]
      - #chapter("dev/udon/udon-vm.typ")[Udon VM]
      - #chapter("dev/udon/udon-variable-table.typ")[Udon Variable Table]
    - #chapter("dev/internals/main.typ")[内部机制]
      - #chapter("dev/internals/overview.typ")[概览]
      - #chapter("dev/internals/extract-udon-module-info.typ")[提取 `UdonModuleInfo.json`]
      - #chapter("dev/internals/bytecode-parser.typ")[反汇编]
      - #chapter("dev/internals/data-flow-analyzer.typ")[数据流分析]
      - #chapter("dev/internals/code-generator.typ")[代码生成]
    - #chapter("dev/contributing.typ")[贡献]
    = 关于
    - #chapter("about/motivation.typ")[动机]
    - #chapter("about/impact.typ")[影响]
    - #chapter("about/me.typ")[关于我]

    // end of summary
  ],
)

#build-meta(dest-dir: "../dist")

// #get-book-meta()

// re-export page template
#import "/docs/_utils/gh-pages.typ": heading-reference, project
#let book-page = project
#let cross-link = cross-link
#let heading-reference = heading-reference
