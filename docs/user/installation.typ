#import "/docs/book.typ": book-page
#import "/docs/_utils/common.typ": *

#show: book-page.with(title: "安装")


For English version, refer to #cross-link("/user-en/installation.typ")[Installation].

本节介绍本项目的安装方式.

= 安装本反编译器

请从本仓库的 #release 页面获取发布版本或者从 #release-workflow 工作流页面获取每日构建版本的可执行文件.

= 获取必要的资源
具体来说, 为了使反编译过程正常进行, 你还需要获取 `UdonModuleInfo.json`. 它是 #udon 程序能调用的一切 C\# 函数的信息. 由于 #vrc-sdk-license 的限制, 我们不能在代码仓库中分发这一文件.

你当然可以从其他渠道取得这一文件, 不过我们建议你通过下面的步骤自行生成该文件

+ 按 VRChat 创作文档的#link("https://creators.vrchat.com/sdk/")[指引], 创建一个安装了 VRChat Base SDK 和 VRChat *World* SDK 的 Unity 项目
+ 使用 #vcc 确认上述两个 SDK 包均已升级到最新版本
+ 在该项目中新建 `Assets/Editor` 目录, 将本项目提供的 #link("https://github.com/ParaN3xus/udon-decompiler/tree/main/Editor")[所有编辑器脚本] 复制到该目录中
+ 在 Unity 的顶部菜单栏点击 `Tools/Extract Udon Module Info`
+ 控制台中应该出现日志(数值可能略有不同)
  ```
  Registry lookup built with 34756 entries.
  Module info saved to: Assets/UdonModuleInfo.json
  Total modules extracted: 772
  ```
  然后可以在所展示目录中找到 `UdonModuleInfo.json`
