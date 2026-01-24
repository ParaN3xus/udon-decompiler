import os
from pathlib import Path

import pytest
from syrupy.data import Snapshot, SnapshotCollection
from syrupy.extensions.single_file import SingleFileSnapshotExtension, WriteMode

from tests.ci.md_cases import load_cases, parse_markdown_cases
from udon_decompiler import (
    decompile_program_to_source,
    ModuleInfoLoader,
    ProgramLoader,
)


class CsSnapshotExtension(SingleFileSnapshotExtension):
    file_extension = "cs"
    snapshot_root: Path | None = None
    _write_mode = WriteMode.TEXT

    @classmethod
    def dirname(cls, *, test_location):
        if cls.snapshot_root is not None:
            return str(cls.snapshot_root)
        return super().dirname(test_location=test_location)

    @classmethod
    def get_snapshot_name(cls, *, test_location, index=0) -> str:
        if isinstance(index, str) and index:
            return index
        return super().get_snapshot_name(test_location=test_location, index=index)

    @classmethod
    def write_snapshot(cls, *, snapshot_location: str, snapshots) -> None:
        if not snapshots:
            return

        snapshot_collection = SnapshotCollection(location=snapshot_location)
        for data, test_location, index in snapshots:
            snapshot_name = cls.get_snapshot_name(
                test_location=test_location, index=index
            )
            snapshot_collection.add(Snapshot(name=snapshot_name, data=data))

        Path(snapshot_location).parent.mkdir(parents=True, exist_ok=True)
        cls.write_snapshot_collection(snapshot_collection=snapshot_collection)


CASES_ROOT = Path("tests/cases")
CASE_PATHS = load_cases(CASES_ROOT)


def _extract_dumped_json(case_path: Path) -> str:
    text = case_path.read_text(encoding="utf-8")
    blocks = parse_markdown_cases(text, case_path)
    if len(blocks) < 2:
        raise ValueError(f"{case_path}: dumped.json block is missing")
    dumped = blocks[1]["content"].strip()
    if not dumped:
        raise ValueError(f"{case_path}: dumped.json block is empty")
    return dumped + "\n"


def _decompile_json_to_source(json_text: str) -> str:
    program = ProgramLoader.load_from_json_string(json_text)
    _, source_code = decompile_program_to_source(program)
    return source_code


@pytest.fixture(scope="session", autouse=True)
def _load_module_info() -> None:
    info_path = Path(os.environ.get("UDON_MODULE_INFO", "local/UdonModuleInfo.json"))
    if not info_path.exists():
        pytest.skip(f"Module info file not found at {info_path}")
    ModuleInfoLoader.load_from_file(info_path)


@pytest.mark.parametrize("case_path", CASE_PATHS, ids=lambda p: str(p))
def test_decompiled_snapshots(case_path: Path, snapshot) -> None:
    if not CASE_PATHS:
        pytest.skip("No markdown cases found in tests/cases")

    snapshot_dir = case_path.parent / "snaps"
    snapshot_dir.mkdir(parents=True, exist_ok=True)
    snapshot_path = snapshot_dir / f"{case_path.stem}.cs"
    if not snapshot_path.exists():
        pytest.skip(f"Snapshot not found at {snapshot_path}")

    try:
        dumped_json = _extract_dumped_json(case_path)
    except ValueError as exc:
        pytest.skip(str(exc))

    try:
        actual = _decompile_json_to_source(dumped_json)
    except Exception as exc:
        pytest.skip(f"{case_path}: failed to load program ({exc})")

    CsSnapshotExtension.snapshot_root = snapshot_dir
    snapshot = snapshot(extension_class=CsSnapshotExtension, name=case_path.stem)
    snapshot.assert_match(actual)
