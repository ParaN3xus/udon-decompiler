#import "/docs/book.typ": book-page
#import "@preview/shiroa:0.3.1": shiroa-sys-target

#show: book-page.with(title: "项目现状")

截至目前, 本项目尚未发布首个正式版本. 本项目仍处于快速迭代阶段, 仍有一些工作可以完成.

= todo
- [ ] Remove `todo:` marks from code
- [ ] `UdonProgramDumper` supporting different `serializedProgramCompressedBytes` formats
- [ ] Tests
  - [ ] Unit tests
  - [ ] E2E tests based on real-world cases: Decompiling well-known open-source UdonSharp projects, such as QvPen.
- [ ] Review all LLM-generated code
- [ ] Better `CSharpCodeGenerator`
  - [ ] Type alias like `bool` for `System.Boolean`
  - [ ] Array access
  - [ ] Comments
  - [ ] Namespaces
