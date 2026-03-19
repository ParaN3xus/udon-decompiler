#import "/docs/book.typ": book-page, cross-link, heading-reference
#import "/docs/_utils/common.typ": *

#show: book-page.with(title: "Usage")

*This document was translated from Chinese by an LLM.*


After installing this project, you can decompile Udon programs in a world.

= Obtain the world file

This project does not provide guidance on this. You should have a `.vrcw` file.

= Extract Udon programs

+ Use the `UdonProgramDumper` program downloaded in the [= Installation] section of #cross-link-heading("/user-en/installation.typ", [= Installation])[Installation] to extract program files from `.vrcw`. Specifically:
- Use the command line
  ```shell-unix-generic
  UdonProgramDumper <world.vrcw>
  ```
  You should get output like this
  ```
  $ UdonProgramDumper example.vrcw
  [example.vrcw] dumped 8 program(s) to example-dumped-programs
  ```
- Drag and drop a `.vrcw` file directly onto `UdonProgramDumper`. This is effectively equivalent to the previous method.

You can then find the generated `dumped-programs` folder in the same directory as the `.vrcw` file.

= Decompile

Run
```shell-unix-generic
udon-decompiler dc dumped-programs --info UdonModuleInfo.json
```
You need to replace `dumped-programs` and `UdonModuleInfo.json` with their actual paths.

The console should show logs like
```shell-unix-generic
2026-03-18T03:15:38.832775Z  INFO udon_decompiler: logging initialized level=info
2026-03-18T03:15:38.832843Z  INFO udon_decompiler: start command mode=Dc input=dumped-programs output=None template=None
2026-03-18T03:15:38.837946Z  INFO udon_decompiler: processing directory mode=Dc input_dir=dumped-programs output=None template=None
2026-03-18T03:15:38.842021Z  INFO udon_decompiler: processing "dumped-programs/27d9de9b1e2d2424cadf167f75a47d24.hex"
2026-03-18T03:15:38.845832Z  INFO udon_decompiler::decompiler::context: class name inferred: Sonic853.Udon.Keypad.UdonVRCheckerObjects
2026-03-18T03:15:38.845873Z  INFO udon_decompiler::decompiler::context: decompile context loaded bytecode_len=468 instruction_count=60 entry_points=1 symbols=15 heap_entries=23
2026-03-18T03:15:38.846975Z  INFO udon_decompiler::decompiler::variable: 23 variables identified from heap
2026-03-18T03:15:39.018584Z  INFO udon_decompiler::decompiler::module_info: successfully loaded module info
2026-03-18T03:15:39.019550Z  INFO udon_decompiler::decompiler::basic_block: 7 basic blocks identified
2026-03-18T03:15:39.019791Z  INFO udon_decompiler::decompiler::cfg: 1 functions discovered with their cfgs built
2026-03-18T03:15:39.019884Z  INFO udon_decompiler::decompiler::ir::builder: 1 IrFunctions built
2026-03-18T03:15:39.020321Z  INFO udon_decompiler::decompiler::transform::pipeline: IR transformed.
2026-03-18T03:15:39.020411Z  INFO udon_decompiler::decompiler::codegen: c# code for UdonVRCheckerObjects generated!
2026-03-18T03:15:39.073924Z  INFO udon_decompiler: dumped-programs/27d9de9b1e2d2424cadf167f75a47d24.hex -> dumped-programs-decompiled/Sonic853_Udon_Keypad_UdonVRCheckerObjects.cs
...
2026-03-18T03:15:39.710051Z  INFO udon_decompiler: done!
```

You can then find all decompilation results, that is, `.cs` pseudocode files, in the `dumped-programs-decompiled` directory.
