import re
from pathlib import Path

CODE_FENCE_RE = re.compile(r"```([^\n]*)\n(.*?)\n```", re.DOTALL)


def load_cases(root: Path) -> list[Path]:
    if not root.exists():
        return []
    return sorted(p for p in root.rglob("*.md") if p.is_file())


def parse_markdown_cases(text: str, path: Path) -> list[dict]:
    blocks = list(CODE_FENCE_RE.finditer(text))
    if len(blocks) not in (1, 2):
        raise ValueError(
            f"{path}: expected 1 or 2 code fences, found {len(blocks)}"
        )
    return [
        {
            "lang": (m.group(1) or "").strip(),
            "content": m.group(2),
            "span": m.span(),
        }
        for m in blocks
    ]
