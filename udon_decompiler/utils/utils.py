from typing import TypeVar

T = TypeVar("T")


def sliding_window(lst: list[T], window: int, step: int = 1) -> list[list[T]]:
    return [lst[i : i + window] for i in range(0, len(lst) - window + 1, step)]


class Singleton(type):
    _instances = {}

    def __call__(cls, *args, **kwargs):
        if cls not in cls._instances:
            cls._instances[cls] = super().__call__(*args, **kwargs)
        return cls._instances[cls]
