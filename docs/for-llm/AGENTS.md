# AGENTS

Quick, repo-specific guidance for automated agents.

## Start here

- Read `docs/for-llm/CONTEXT.md` first.
- Skim `docs/dev/internals/overview.typ` for the human-facing architecture overview.

## Project rules of thumb

- Do not hand-edit dumped JSON blocks in `tests/cases/**.md`; CI regenerates them.
- Keep new files ASCII when possible.
- Follow existing module boundaries (`loaders/`, `parsers/`, `analysis/`, `codegen/`).
- Control-flow structuring is implemented via the IR transform pipeline (not legacy SCFG lifters).

## Key commands

- Install deps (dev): `uv sync --group dev`
- Run lint: `uv run ruff check .`
- Type check: `uv run pyright`
- Run tests: `uv run pytest -q -vv`
- Filtered tests: `uv run pytest -k "<expr>" -vv`
- Update snapshots: `uv run pytest --snapshot-update`
- Decompile one markdown case and print GT/decompiled output:
  `python -m tests.decompile_case tests/cases/control_flow/SwitchCaseLong.md`
- Enable debug log for the single-case helper: add `--debug-log`

## Testing prerequisites

- `UdonModuleInfo.json` must exist, or set `UDON_MODULE_INFO` to its path.

## Typical test-case update flow

1. Add/update C# in `tests/cases/**.md` (first fenced block).
2. Push so `compile-test-cases.yml` refreshes dumped JSON blocks.
3. Pull the updated markdown, then run `uv run pytest --snapshot-update`.
4. Review generated snapshots and commit.

## Workflows to know

- `compile-test-cases.yml`: compiles markdown cases in Unity and writes dumped JSON back.
- `ci.yml`: `uv sync --group dev --frozen`, ruff, pyright, pytest.
- `docs.yml`: builds Typst docs with shiroa and deploys to Cloudflare Pages.
- `generate-udon-module-info.yml`: extracts `UdonModuleInfo.json` in Unity and uploads artifact.
