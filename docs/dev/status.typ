#import "/docs/book.typ": book-page
#import "@preview/shiroa:0.3.1": shiroa-sys-target

#show: book-page.with(title: "项目现状")


截至目前, 本项目尚未发布首个正式版本. 本项目仍处于快速迭代阶段, 仍有一些工作可以完成.

= todo
- [ ] Remove `todo:` marks from code
- [ ] Tests
  - [-] Unit tests
  - [ ] E2E tests based on real-world cases: Decompiling well-known open-source UdonSharp projects, such as QvPen.
    - [x] QvPen
    - [ ] Pick other well-known projects from https://github.com/topics/udon.
- [ ] Review all LLM-generated code
  - [ ] `src/odin`
  - [ ] `src/udon_asm`
  - [ ] `src/decompiler/transform/passes`
- [ ] More renderers & parsers of C\# objects
- [ ] Editor script, IR and code generator support of namespaces
- [ ] Type alias like `bool` for `System.Boolean`: #link("https://github.com/icsharpcode/ILSpy/blob/2f311c233d301f5d622c213b7c2abcecb1fcc217/ICSharpCode.Decompiler/Disassembler/DisassemblerHelpers.cs#L342")[reference]
- [ ] IR and code generator support of array access
- [ ] Add comment hints for code sections that failed to decompile
- [ ] Docs for disassembler and assembler feature
- [ ] Better dynamic analysis
  - [ ] Enrich the Heap simulator model to support bounded simulation
  - [ ] A C\# engine to simulate the execution of some native C\# functions
  - [ ] Simulate the same basic block separately for each distinct entry point, then merge
- [ ] VPM reposity for editor scripts
