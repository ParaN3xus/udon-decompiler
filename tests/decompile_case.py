import argparse
import logging
import os
import sys
from pathlib import Path

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tests.test_snapshots import (
    _decompile_json_to_source,
    _extract_case_source_and_dumped_json,
)
from udon_decompiler import ModuleInfoLoader
from udon_decompiler.utils.logger import set_logger_level


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Decompile one markdown case and print GT/decompiled source."
    )
    parser.add_argument("case_path", help="Path to markdown case file.")
    parser.add_argument(
        "--debug-log",
        action="store_true",
        help="Enable DEBUG logs during decompilation.",
    )
    return parser.parse_args()


def main() -> None:
    args = _parse_args()

    if args.debug_log:
        set_logger_level(logging.DEBUG)

    info_path = Path(os.environ.get("UDON_MODULE_INFO", "UdonModuleInfo.json"))
    ModuleInfoLoader.load_from_file(info_path)

    gt_source, dumped_json = _extract_case_source_and_dumped_json(Path(args.case_path))
    actual = _decompile_json_to_source(dumped_json)

    print("-------- GT Source --------")
    print(gt_source)
    print("-------- Decompiled Source --------")
    print(actual)


if __name__ == "__main__":
    main()
