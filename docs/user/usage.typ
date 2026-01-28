#import "/docs/book.typ": book-page, cross-link, heading-reference
#import "/docs/_utils/common.typ": *

#show: book-page.with(title: "使用")

完成本项目的安装后, 你就可以反编译世界中的 Udon 程序了.

= 获取世界文件

本项目不提供这方面的指引. 你应当有一个 `.vrcw` 格式的文件.

= Dump SerializedUdonPrograms

+ 使用 #asset-ripper 解包世界文件(导出项目). 无需使用 #asset-ripper 的 Premium Edition, 它对反编译 Udon 程序没有额外的增益.
+ 找到导出项目的 `ExportedProject/Assets/MonoBehaviour` 文件夹, 这里面有所有可能是 UdonProgram 的资产文件
+ #cross-link-heading(
    "/user/installation.typ",
    [= 获取必要的资源],
  )[获取必要的资源]一节创建了一个安装了本项目提供的编辑器脚本的项目. 打开该项目, 在 Unity 的顶部菜单栏点击 `Tools/Udon Program Dumper`. 在弹出的窗口中, 有一个标有 "Folder Path" 的输入框. 在该输入框中输入 `ExportedProject/Assets/MonoBehaviour` 文件夹的路径, 点击 "Dump All .asset Files" 按钮
+ 控制台中应该出现日志
  ```
  Generated: ExportedProject/Assets/MonoBehaviour/dumped/xxx.asset.json
  ...
  Dumped!
  ```
  然后可以在 `ExportedProject/Assets/MonoBehaviour/dumped` 目录中找到一些 `.json` 文件. 这些是 dump 出的 Udon 程序, 是本项目主体程序(下称"本反编译器")的输入

= 反编译

运行
```shell-unix-generic
udon-decompiler ExportedProject/Assets/MonoBehaviour/dumped --info UdonModuleInfo.json
```
此处需要填写 `dumped` 文件夹和 `UdonModuleInfo.json` 文件的实际路径.

控制台应当显示日志如
```shell-unix-generic
2026-01-27 16:00:43 - udon_decompiler - INFO - Decompiling xxx.asset.json
2026-01-27 16:00:43 - udon_decompiler - INFO - Successfully loaded program: UdonProgram(
  bytecode_length=460,
  symbols=25,
  entry_points=4,
  heap_entries=34
)
2026-01-27 16:00:43 - udon_decompiler - INFO - Parsed 61 instructions
2026-01-27 16:00:43 - udon_decompiler - INFO - Created 7 basic blocks
2026-01-27 16:00:43 - udon_decompiler - INFO - Built 4 control flow graphs
2026-01-27 16:00:43 - udon_decompiler - INFO - Program entry points identified: [EntryPoint(_start @ 0x00000008), EntryPoint(_f @ 0x00000058), EntryPoint(_g @ 0x00000094), EntryPoint(_h @ 0x00000178)]
2026-01-27 16:00:43 - udon_decompiler - INFO - Completed dataflow analysis for 4 functions
2026-01-27 16:00:43 - udon_decompiler - INFO - Generating C# code for yyy...
2026-01-27 16:00:43 - udon_decompiler - INFO - Decompiled: xxx.asset.json -> dumped-decompiled/yyy.cs
...
2026-01-27 16:00:43 - udon_decompiler - INFO - Done.
```

然后可以在 `ExportedProject/Assets/MonoBehaviour/dumped-decompiled` 目录中找到所有反编译结果, 也即 `.cs` 格式的伪代码.
