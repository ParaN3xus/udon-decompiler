#import "/docs/book.typ": book-page
#import "@preview/shiroa:0.3.1": shiroa-sys-target

#show: book-page.with(title: "动机")

关于动机, 我没有什么漂亮话能说. 我开发这个项目就是*为了看到一些 VRChat 世界内部的逻辑*, 从而*理解*, *破解*, *修改*世界.

开始编码前, 我确实在网络上找到了开源的#link("https://github.com/UdonSharpRE/UdonSharpDecompiler")[反汇编器]和#link("https://github.com/extremeblackliu/UdonSharpDisassembler")[反编译器]. 但是我对这两者的效果都不太满意, 而且我自认为有更好的思路, 因此我还是决定自己写一个新的. 本项目的实现过程没有参考 #link("https://github.com/UdonSharpRE")[UdonSharpRE] 组织的任何仓库和上述反汇编器仓库.

关于开源, 我其实并没有想太多. 我只是觉得既然写了, 那就开源吧, 仅此而已.

话虽如此, 至今我还没看我最初想要理解的那个世界的反编译代码, 或许我也不会看了. 今后我在此项目上投入的精力可能更多地取决于社区对此的反馈和我个人的其他需求. 或许某一天我会把本项目的仓库归档, 或许会转移给其他人, who knows.
