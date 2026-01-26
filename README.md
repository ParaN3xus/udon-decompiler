# Udon Decompiler

The UdonSharp Decompiler.

## Usage

1. Extract `MonoBehaviour`s from VRChat worlds with [AssetRipper](https://github.com/AssetRipper/AssetRipper)
2. Create a new VRChat World Project, and import all extracted `MonoBehaviour`s
3. Use the [UdonProgramDumper](https://github.com/ParaN3xus/udon-decompiler/blob/main/Editor/UdonProgramDumperGUI.cs) to dump potential `UdonProgram`s in `MonoBehaviour`s into JSON
4. Obtain `UdonModuleInfo.json`:
   - Recommended: create a local VRChat World Project and run `Tools/Extract Udon Module Info` (see `Editor/UdonModuleInfoExtractor.cs`).
   - Not recommended: fork this repo and run the `Generate UdonModuleInfo.json` GitHub Action (requires Unity license/activation).
5. `python -m udon_decompiler <input> [--info <UdonModuleInfo.json>]`
6. Enjoy!

## TODO

- [ ] Remove `todo:` marks from code
- [ ] `UdonProgramDumper` supporting different `serializedProgramCompressedBytes` formats
- [ ] Tests
  - [ ] Unit tests
  - [ ] E2E tests based on real-world cases: Decompiling well-known open-source UdonSharp projects, such as QvPen.
- [ ] Review all LLM-generated code
- [ ] Better `CSharpCodeGenerator`
  - [ ] Type alias like `bool` for `System.Boolean`
  - [ ] Array access
  - [ ] `out` keyword
  - [ ] Comments
  - [ ] Flatten nested `if-else` statements.
  - [ ] Namespaces

## Contributing

I know very little about decompilation, and most of the work was completed with the assistance of LLMs. Professional help is welcome and eagerly needed.

We welcome any contribution to the project, including bug reports and fixes, feature requests or additions, code refactoring, and completing the TODOs listed above.

## License

AGPL-3.0-only.
