#import "/docs/book.typ": book-page, cross-link, heading-reference
#import "/docs/_utils/common.typ": *

#show: book-page.with(title: "Usage")

This document was translated from Chinese by an LLM.

After installing this project, you can decompile Udon programs in a world.

= Obtain the world file

This project does not provide guidance on this. You should have a `.vrcw` file.

= Dump SerializedUdonPrograms

+ Use #asset-ripper to unpack the world file (export the project). The Premium Edition of #asset-ripper is not required and provides no extra benefit for decompiling Udon programs.
+ Find the `ExportedProject/Assets/MonoBehaviour` folder in the exported project; it contains all assets that may be UdonPrograms
+ The [= Obtain required resources] section of #cross-link-heading(
    "/user-en/installation.typ",
    [= Obtain required resources],
  )[Obtain required resources] created a project with the editor scripts from this project installed. Open that project and click `Tools/Udon Program Dumper` in Unity's top menu bar. In the popup window, enter the path to `ExportedProject/Assets/MonoBehaviour` in the input box labeled "Folder Path", then click "Dump All .asset Files"
+ The console should show logs
  ```
  Generated: ExportedProject/Assets/MonoBehaviour/dumped/xxx.asset.json
  ...
  Dumped!
  ```
  Then you can find some `.json` files under `ExportedProject/Assets/MonoBehaviour/dumped`. These are the dumped Udon programs and are the input to the main program of this project (hereafter "the decompiler")

= Decompile

Run
```shell-unix-generic
udon-decompiler ExportedProject/Assets/MonoBehaviour/dumped --info UdonModuleInfo.json
```
You need to fill in the actual paths to the `dumped` folder and the `UdonModuleInfo.json` file.

The console should show logs like
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

Then you can find all decompilation results, i.e. `.cs` pseudocode files, in `ExportedProject/Assets/MonoBehaviour/dumped-decompiled`.
