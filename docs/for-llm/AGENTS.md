# AGENTS

Quick repository-specific guidance for automated agents.

## Read First

- Read [`docs/for-llm/CONTEXT.md`](./CONTEXT.md) first.
- If you need the repository-wide collaboration rules, also read the root [`AGENTS.md`](../../AGENTS.md).

## Project Status

- The Rust implementation is the mainline implementation.
- The CLI entry point is [`src/main.rs`](../../src/main.rs).
- The library entry point is [`src/lib.rs`](../../src/lib.rs).

## Current Inputs and Outputs

- `dc` and `dasm` accept:
  - compressed `.hex`
  - Unity `.asset`
- `asm` accepts `.asm` and emits compressed `.hex`
- Program inputs inside `tests/cases/**/*.md` now use fenced `hex` blocks

## Key Commands

- Show CLI help: `cargo run -- --help`
- Decompile one program: `cargo run -- dc <input.hex|input.asset>`
- Disassemble one program: `cargo run -- dasm <input.hex|input.asset>`
- Re-assemble from ASM: `cargo run -- asm <input.asm> --template <original.hex|original.asset>`
- Run e2e smoke with visible progress: `cargo test --test e2e e2e_smoke -- --nocapture`
- Run the full test suite: `cargo test -q`
- Dump per-pass IR for a markdown case:
  `cargo run --bin dump_rust_ir_passes -- --case <case.md>`

## Runtime Prerequisite

- The project expects `UdonModuleInfo.json` in the current working directory by default.
- To override it, use `--module-info <path>`.

Without module info, CFG construction, stack simulation, and extern resolution will fail.

## Layering Rules

- `src/odin/*`
  - Binary Udon program model parsing, serialization, and editing
- `src/udon_asm/*`
  - ASM text <-> program / bytecode
  - heap literal parsing and rendering
  - disassembly bind/comment analysis
- `src/decompiler/*`
  - the decompiler proper
  - `context.rs`: program context loading
  - `cfg.rs` / `basic_block.rs`: basic blocks, CFG, stack simulation, hidden entry discovery
  - `ir/*`: IR nodes and IR builder
  - `transform/*`: structured control-flow recovery and IR cleanup
  - `codegen.rs`: C# generation

Do not push cross-layer logic into `main.rs` or `util`.

## Practical Rules of Thumb

- If you are changing control-flow recovery:
  - start with `src/decompiler/cfg.rs`
  - then inspect `src/decompiler/transform/passes/*`
- If you are changing jump/call recognition:
  - CFG-stage logic lives in `src/decompiler/cfg.rs`
  - IR-stage logic lives in `src/decompiler/ir/builder.rs`
  - `dasm` bind/comment logic lives in `src/udon_asm/analysis.rs`
- If you are changing extern behavior:
  - inspect `src/decompiler/module_info.rs`
  - `src/decompiler/ir/builder.rs`
  - `src/decompiler/codegen.rs`
- If you are changing input formats:
  - `.hex`: `src/util/hex.rs`
  - `.asset`: `src/util/asset.rs`

## Test Case Workflow

1. Update the first fenced block (ground-truth C#) in `tests/cases/**/*.md`.
2. Keep the program input block as fenced `hex`.
3. Run `cargo test --test e2e e2e_smoke -- --nocapture` first.
4. If snapshots are involved, inspect `.snap.new` files carefully instead of blindly accepting them.

## Notes

- `docs/for-llm/*` is agent-facing repository documentation, not end-user documentation.
- If the repository structure, default pipeline order, or program input format changes again, update these files too.
