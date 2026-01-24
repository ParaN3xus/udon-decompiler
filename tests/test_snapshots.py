import os
import tempfile
from pathlib import Path

import pytest
from syrupy.extensions.single_file import SingleFileSnapshotExtension

from tests.ci.md_cases import load_cases, parse_markdown_cases
from udon_decompiler import (
    BytecodeParser,
    DataFlowAnalyzer,
    ModuleInfoLoader,
    ProgramCodeGenerator,
    ProgramLoader,
    UdonModuleInfo,
)


class CsSnapshotExtension(SingleFileSnapshotExtension):
    _file_extension = "cs"


CASES_ROOT = Path("tests/cases")
CASE_PATHS = load_cases(CASES_ROOT)


def _extract_dumped_json(case_path: Path) -> str:
    text = case_path.read_text(encoding="utf-8")
    blocks = parse_markdown_cases(text, case_path)
    dumped = blocks[1]["content"].strip()
    if not dumped:
        raise ValueError(f"{case_path}: dumped.json block is empty")
    return dumped + "\n"


def _decompile_json_to_source(json_text: str) -> str:
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as tmp:
        tmp.write(json_text)
        tmp_path = Path(tmp.name)

    try:
        program = ProgramLoader.load_from_file(str(tmp_path))
        bc_parser = BytecodeParser(program)
        instructions = bc_parser.parse()
        analyzer = DataFlowAnalyzer(program, UdonModuleInfo(), instructions)
        function_analyzers = analyzer.analyze()
        _, source_code = ProgramCodeGenerator.generate_program(
            program, function_analyzers
        )
        return source_code
    finally:
        tmp_path.unlink(missing_ok=True)


@pytest.fixture(scope="session", autouse=True)
def _load_module_info() -> None:
    info_path = Path(os.environ.get("UDON_MODULE_INFO", "local/UdonModuleInfo.json"))
    if not info_path.exists():
        pytest.skip(f"Module info file not found at {info_path}")
    ModuleInfoLoader.load_from_file(info_path)


@pytest.mark.parametrize("case_path", CASE_PATHS, ids=lambda p: str(p))
def test_decompiled_snapshots(case_path: Path, snapshot, snapshot_settings) -> None:
    if not CASE_PATHS:
        pytest.skip("No markdown cases found in tests/cases")

    dumped_json = _extract_dumped_json(case_path)
    actual = _decompile_json_to_source(dumped_json)

    snapshot_settings.extension_class = CsSnapshotExtension
    snapshot_settings.snapshot_dir = case_path.parent / "snaps"
    snapshot_settings.snapshot_dir.mkdir(parents=True, exist_ok=True)

    snapshot.assert_match(actual, case_path.stem)
