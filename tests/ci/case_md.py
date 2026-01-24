#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

from tests.ci.md_cases import load_cases, parse_markdown_cases


def build_input(cases_root: Path, output_path: Path) -> None:
    cases = load_cases(cases_root)
    if not cases:
        raise SystemExit(f"No case markdown files found under {cases_root}")

    requests = []
    for case_path in cases:
        text = case_path.read_text(encoding="utf-8")
        blocks = parse_markdown_cases(text, case_path)
        source_code = blocks[0]["content"]
        requests.append(
            {
                "className": case_path.stem,
                "sourcePath": str(case_path),
                "sourceCode": source_code,
            }
        )

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        json.dumps({"requests": requests}, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


def update_cases(input_path: Path, output_path: Path) -> None:
    input_data = json.loads(input_path.read_text(encoding="utf-8"))
    output_data = json.loads(output_path.read_text(encoding="utf-8"))

    requests = input_data.get("requests", [])
    results = output_data.get("results", [])
    error = output_data.get("error")

    if error:
        raise SystemExit(f"Compilation error: {error}")

    if len(requests) != len(results):
        raise SystemExit(
            f"Request/result count mismatch: {len(requests)} "
            f"requests vs {len(results)} results"
        )

    for request, dumped in zip(requests, results):
        case_path = Path(request["sourcePath"])
        text = case_path.read_text(encoding="utf-8")
        blocks = parse_markdown_cases(text, case_path)

        lang = blocks[1]["lang"] or "json"
        dumped_text = dumped.rstrip("\n") + "\n"
        new_block = f"```{lang}\n{dumped_text}```"

        start, end = blocks[1]["span"]
        updated = text[:start] + new_block + text[end:]
        case_path.write_text(updated, encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Manage markdown test cases with C# + dumped.json blocks."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    build_parser = subparsers.add_parser(
        "build-input", help="Create input.json from cases markdown."
    )
    build_parser.add_argument("--cases", type=Path, default=Path("tests/cases"))
    build_parser.add_argument("--output", type=Path, required=True)

    update_parser = subparsers.add_parser(
        "update-md", help="Update markdown dumped.json blocks."
    )
    update_parser.add_argument("--input", type=Path, required=True)
    update_parser.add_argument("--output", type=Path, required=True)

    args = parser.parse_args()
    if args.command == "build-input":
        build_input(args.cases, args.output)
    elif args.command == "update-md":
        update_cases(args.input, args.output)


if __name__ == "__main__":
    main()
