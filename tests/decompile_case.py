import os
import sys
from pathlib import Path

from tests.test_snapshots import _decompile_json_to_source, _extract_dumped_json
from udon_decompiler import ModuleInfoLoader

info_path = Path(os.environ.get("UDON_MODULE_INFO", "UdonModuleInfo.json"))
ModuleInfoLoader.load_from_file(info_path)


case_name = sys.argv[1]

dumped_json = _extract_dumped_json(Path(case_name))
actual = _decompile_json_to_source(dumped_json)

print("-------- Decompiled Source --------")
print(actual)
