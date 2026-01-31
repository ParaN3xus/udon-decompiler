#import "/docs/book.typ": book-page
#import "/docs/_utils/common.typ": *

#show: book-page.with(title: "Installation")

This document was translated from Chinese by an LLM.

This section describes how to install this project.

= Install the decompiler

+ We use #uv for Python project management, so before installing this decompiler, please follow the #uv official #link("https://docs.astral.sh/uv/getting-started/installation/")[guide] to install #uv
+ Install this project as a tool
  ```shell-unix-generic
  uv tool install git+https://github.com/paran3xus/udon-decompiler.git
  ```

= Obtain required resources
More specifically, to make decompilation work, you also need `UdonModuleInfo.json`. It contains information about all C\# functions that #udon programs can call. Due to the #vrc-sdk-license restrictions, we cannot distribute this file in the repository.

You can get this file from other sources, but since later steps reuse the project created here, we recommend generating it yourself with the following steps.

+ Follow the VRChat creator documentation #link("https://creators.vrchat.com/sdk/")[guide] to create a Unity project with both VRChat Base SDK and VRChat *World* SDK installed
+ Use #vcc to ensure both SDK packages above are upgraded to the latest versions
+ Create an `Assets/Editor` directory in that project and copy all editor scripts from this project #link("https://github.com/ParaN3xus/udon-decompiler/tree/main/Editor")[here] into that directory
+ In Unity's top menu bar, click `Tools/Extract Udon Module Info`
+ The console should show logs like (numbers may differ)
  ```
  Registry lookup built with 34756 entries.
  Module info saved to: Assets/UdonModuleInfo.json
  Total modules extracted: 772
  ```
  Then you can find `UdonModuleInfo.json` in the shown directory
