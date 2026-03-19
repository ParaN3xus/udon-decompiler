#import "/docs/book.typ": book-page, cross-link, heading-reference
#import "/docs/_utils/common.typ": *

#show: book-page.with(title: "使用")


For English version, refer to #cross-link("/user-en/usage.typ")[Usage].

完成本项目的安装后, 你就可以反编译世界中的 Udon 程序了.

= 获取世界文件

本项目不提供这方面的指引. 你应当有一个 `.vrcw` 格式的文件.

= 提取 Udon 程序

使用 #cross-link-heading("/user/installation.typ", [= 安装])[安装]一节下载的 `UdonProgramDumper` 程序提取 `.vrcw` 中的程序文件, 具体而言:
- 使用命令行
  ```shell-unix-generic
  UdonProgramDumper dump <world.vrcw>
  ```
  你应该得到类似这样的输出
  ```shell-unix-generic
  $ UdonProgramDumper dump example.vrcw
  [example.vrcw] dumped 8 program(s), 8 public var file(s) to /path/to/example-dumped
  ```
- 直接将 `.vrcw` 文件拖放到 `UdonProgramDumper` 程序上. 这实际上等价于前一种方法.

然后可以在 `.vrcw` 文件相同目录下找到所生成的 `example-dumped` 文件夹, 其中包含:
- `programs/`: 提取出的 `.hex` 程序文件
- `vars/`: 提取出的 `.b64` public var 文件
- `program-var-map.json`: program 与使用它的 GameObject/public var 文件之间的映射

= 反编译

运行
```shell-unix-generic
udon-decompiler dc example-dumped/programs --info UdonModuleInfo.json
```
此处需要填写 `example-dumped/programs` 文件夹和 `UdonModuleInfo.json` 文件的实际路径.

控制台应当显示日志如
```shell-unix-generic
2026-03-18T03:15:38.832775Z  INFO udon_decompiler: logging initialized level=info
2026-03-18T03:15:38.832843Z  INFO udon_decompiler: start command mode=Dc input=example-dumped/programs output=None template=None
2026-03-18T03:15:38.837946Z  INFO udon_decompiler: processing directory mode=Dc input_dir=example-dumped/programs output=None template=None
2026-03-18T03:15:38.842021Z  INFO udon_decompiler: processing "example-dumped/programs/27d9de9b1e2d2424cadf167f75a47d24.hex"
2026-03-18T03:15:38.845832Z  INFO udon_decompiler::decompiler::context: class name inferred: Sonic853.Udon.Keypad.UdonVRCheckerObjects
2026-03-18T03:15:38.845873Z  INFO udon_decompiler::decompiler::context: decompile context loaded bytecode_len=468 instruction_count=60 entry_points=1 symbols=15 heap_entries=23
2026-03-18T03:15:38.846975Z  INFO udon_decompiler::decompiler::variable: 23 variables identified from heap
2026-03-18T03:15:39.018584Z  INFO udon_decompiler::decompiler::module_info: successfully loaded module info
2026-03-18T03:15:39.019550Z  INFO udon_decompiler::decompiler::basic_block: 7 basic blocks identified
2026-03-18T03:15:39.019791Z  INFO udon_decompiler::decompiler::cfg: 1 functions discovered with their cfgs built
2026-03-18T03:15:39.019884Z  INFO udon_decompiler::decompiler::ir::builder: 1 IrFunctions built
2026-03-18T03:15:39.020321Z  INFO udon_decompiler::decompiler::transform::pipeline: IR transformed.
2026-03-18T03:15:39.020411Z  INFO udon_decompiler::decompiler::codegen: c# code for UdonVRCheckerObjects generated!
2026-03-18T03:15:39.073924Z  INFO udon_decompiler: example-dumped/programs/27d9de9b1e2d2424cadf167f75a47d24.hex -> example-dumped/programs-decompiled/Sonic853_Udon_Keypad_UdonVRCheckerObjects.cs
...
2026-03-18T03:15:39.710051Z  INFO udon_decompiler: done!
```

然后可以在 `example-dumped/programs-decompiled` 目录中找到所有反编译结果, 也即 `.cs` 格式的伪代码.
