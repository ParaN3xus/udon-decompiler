# Udon Decompiler Rust Context

## Summary

- Project name: `udon-decompiler`
- Main implementation language: Rust
- Shape: library + CLI
- Goals:
  - decompile Udon programs into readable pseudo-C#
  - provide a reversible `program <-> asm` workflow

## Current High-Level Flow

1. Read an input program
   - `.hex`: compressed hex -> gunzip -> program bytes
   - `.asset`: extract `serializedProgramCompressedBytes` -> gunzip -> program bytes
2. Parse the program with `UdonProgramBinary`
3. Build a `DecompileContext`
4. Identify variables, basic blocks, and function entries
5. Build CFGs and run stack simulation
6. Build IR functions from CFGs
7. Run the transform pipeline
8. Build the IR class
9. Generate and format C#

## Important Entry Points

- CLI: [`src/main.rs`](../../src/main.rs)
- Library exports: [`src/lib.rs`](../../src/lib.rs)
- Decompiler pipeline entry points:
  - `run_analysis_pipeline`
  - `run_decompile_pipeline`
  - `DecompileContext::run_analysis`
  - `DecompileContext::run_decompile`

## Current CLI Semantics

### `dc`

- Input: `.hex` or `.asset`
- Output: `.cs`

### `dasm`

- Input: `.hex` or `.asset`
- Output: `.asm`
- Also emits:
  - `bind`
  - `bind-table`
  - instruction comments for COPY / JUMP_IF_FALSE / internal call / EXTERN

### `asm`

- Input: `.asm`
- Template: original `.hex` or `.asset`
- Output: compressed `.hex`

## Input Formats

### `.hex`

- The file content is a hex string of gzip-compressed program bytes.
- Read/write logic lives in [`src/util/hex.rs`](../../src/util/hex.rs).

### `.asset`

- Parsed with `unity-asset-yaml`
- Reads `serializedProgramCompressedBytes`
- Supports both:
  - a single-line hex string
  - a YAML byte list
- Logic lives in [`src/util/asset.rs`](../../src/util/asset.rs).

## Repository Layout

### `src/odin/*`

- Low-level Udon program binary model
- Parsing, serialization, and editing all live here
- Core type: `UdonProgramBinary`

### `src/udon_asm/*`

- ASM parsing: `parse.rs`
- Disassembly: `disassemble.rs`
- Re-assembly / patching: `apply.rs`
- Bytecode codec: `codec.rs`
- Heap literal parsing/rendering: `literal/*`
- ASM analysis (binds/comments): `analysis.rs`

### `src/decompiler/*`

- `context.rs`
  - loads heap / symbols / entries / instructions into `DecompileContext`
- `basic_block.rs`
  - basic block identification
- `cfg.rs`
  - CFG construction, stack simulation, switch scaffold handling, hidden entry discovery
- `ir/*`
  - IR nodes, IR builder, control-flow graph helpers, dominance helpers
- `transform/*`
  - IL transforms and program transforms
- `codegen.rs`
  - IR -> pseudo-C#
- `pipeline.rs`
  - analysis and decompile pipeline orchestration

## Default Transform Order

IL transforms:

1. `ControlFlowSimplification`
2. `ConstToLiteral`
3. `TempVariableInline`
4. `DetectExitPoints(false)`
5. `LoopDetection`
6. `DetectExitPoints(true)`
7. `ConditionDetection`
8. `HighLevelLoopTransform`
9. `HighLevelSwitchTransform`
10. `HighLevelLoopStatementTransform`
11. `StructuredControlFlowCleanupTransform`
12. `CollectLabelUsage`
13. `CollectVariables`

Program transforms:

1. `IrClassConstructionTransform`
2. `PromoteGlobals`

## Function Entry and Internal Call Recognition

There are two related but distinct stages.

### CFG Stage

Located in [`src/decompiler/cfg.rs`](../../src/decompiler/cfg.rs).

Responsibilities:

- hidden entry discovery
- distinguishing returning calls from tail-call-like jumps
- splitting function CFGs

### IR Builder Stage

Located in [`src/decompiler/ir/builder.rs`](../../src/decompiler/ir/builder.rs).

Responsibilities:

- converting a `JUMP` into either:
  - a normal `Jump`
  - an `InternalCall`
  - an `InternalCall + Return` tail-call shape

Do not update only one side. If CFG discovery and IR call recognition diverge, you will get cases where a hidden function exists but jumps to it still decompile as raw gotos.

## Current Codegen Behavior

- `codegen` now tries to print IR mechanically
- labels are rendered as `bb_xxxxxxxx`
- `InternalCall + Return` emits as:

```csharp
function_x();
return;
```

## Tests

- Cases: `tests/cases/**/*.md`
- Each case normally contains:
  - first fenced block: ground-truth C#
  - second fenced block: program input in fenced `hex`
- e2e test entry point: [`tests/e2e.rs`](../../tests/e2e.rs)
  - `e2e_smoke`
  - `e2e_snapshot`

## Current e2e Behavior

- `e2e` reads fenced `hex`
- each case prints a progress line when run with `-- --nocapture`
- missing `hex` fences are treated as test failures, not skipped cases

## Useful Debugging Flows

### Inspect one program as ASM

```bash
cargo run -- dasm <input.asset> /tmp/out.asm
```

### Inspect one program as C#

```bash
cargo run -- dc <input.asset> /tmp/out.cs
```

### Inspect per-pass IR for one markdown case

```bash
cargo run --bin dump_rust_ir_passes -- --case tests/cases/control_flow/SwitchCaseLong.md
```

## Runtime Prerequisite

- The project expects `UdonModuleInfo.json` in the current working directory by default.
- Without it:
  - extern semantics cannot be resolved correctly during CFG/stack simulation
  - `dc`, `dasm`, and `e2e` may fail
