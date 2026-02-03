import re
from pathlib import Path

CODE_FENCE_RE = re.compile(r"```([^\n]*)\n(.*?)\n```", re.DOTALL)
DIRECTIVE_RE = re.compile(r"<!--\s*ci\s*:\s*([^>]*)-->", re.IGNORECASE)

SKIP_COMPILE_DIRECTIVE = "skip-compile"


def load_cases(root: Path) -> list[Path]:
    if not root.exists():
        return []
    return sorted(
        p for p in root.rglob("*.md") if p.is_file() and p.name != "README.md"
    )


def parse_markdown_cases(text: str, path: Path) -> list[dict]:
    blocks = list(CODE_FENCE_RE.finditer(text))
    if len(blocks) not in (1, 2):
        raise ValueError(f"{path}: expected 1 or 2 code fences, found {len(blocks)}")
    return [
        {
            "lang": (m.group(1) or "").strip(),
            "content": m.group(2),
            "span": m.span(),
        }
        for m in blocks
    ]


def parse_case_directives(text: str) -> set[str]:
    directives: set[str] = set()
    for match in DIRECTIVE_RE.finditer(text):
        raw = match.group(1)
        for token in re.split(r"[,\s]+", raw.strip()):
            if token:
                directives.add(token.lower())
    return directives
