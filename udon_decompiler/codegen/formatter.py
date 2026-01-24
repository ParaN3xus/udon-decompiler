import re


class CodeFormatter:
    def __init__(self):
        self.indent_size = 4

    def format(self, code: str) -> str:
        lines = code.split("\n")
        formatted_lines = []

        current_indent = 0
        in_case_body = False

        for line in lines:
            stripped = line.strip()

            if not stripped:
                formatted_lines.append("")
                continue

            is_case = stripped.startswith("case ") or stripped == "default:"
            if in_case_body and (is_case or stripped.startswith("}")):
                current_indent = max(0, current_indent - 1)

            indent_change_before = self._get_indent_change_before(stripped)
            indent_change_after = self._get_indent_change_after(stripped)

            current_indent += indent_change_before
            current_indent = max(0, current_indent)

            indent_str = " " * (current_indent * self.indent_size)
            formatted_lines.append(indent_str + stripped)

            current_indent += indent_change_after
            current_indent = max(0, current_indent)

            if stripped.startswith("}"):
                in_case_body = False
            elif is_case:
                in_case_body = True

        return "\n".join(formatted_lines)

    def _get_indent_change_before(self, line: str) -> int:
        if line.startswith("}"):
            return -1

        return 0

    def _get_indent_change_after(self, line: str) -> int:
        if line.endswith("{"):
            return 1

        if line.startswith("case ") and line.endswith(":"):
            return 1

        if line == "default:":
            return 1

        return 0

    def remove_empty_lines(self, code: str) -> str:
        lines = code.split("\n")
        result = []

        prev_empty = False
        for line in lines:
            is_empty = not line.strip()

            if is_empty and prev_empty:
                continue

            result.append(line)
            prev_empty = is_empty

        return "\n".join(result)

    def add_spacing(self, code: str) -> str:
        code = re.sub(r"([^=!<>])=([^=])", r"\1 = \2", code)
        code = re.sub(r"([^<>!])([<>]=?|[!=]=)", r"\1 \2", code)

        code = re.sub(r",([^\s])", r", \1", code)

        return code
