from typing import TypeVar

T = TypeVar("T")


def sliding_window(lst: list[T], window: int, step: int = 1) -> list[list[T]]:
    return [lst[i : i + window] for i in range(0, len(lst) - window + 1, step)]
