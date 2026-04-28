#!/usr/bin/env python3

"""
Post-processing of generated code.
"""

from collections.abc import Generator, Callable
from pathlib import Path
from textwrap import dedent
from typing import TypeAlias
import logging


FileModifier: TypeAlias = Callable[[list[str]], Generator[str, None, None]]
INDENT = "    "


def file_modifications() -> Generator[tuple[Path, FileModifier], None, None]:
    """Return a generator of file paths and their corresponding modification functions."""

    yield Path("types/ObjectParamAPI.ts"), object_param_api_ts
    yield Path("types/PromiseAPI.ts"), promise_api_ts
    yield Path("models/ObjectSerializer.ts"), object_serializer_ts


def main():
    """Main function to perform file modifications."""

    logging.basicConfig(level=logging.INFO, format="[%(levelname)s] %(message)s")

    subdir = Path("typescript")
    for file_path, modify_fn in file_modifications():
        logging.info("Modifying %s…", file_path)

        file_path = subdir / file_path

        try:
            with open(file_path, "r", encoding="utf-8") as f:
                file_contents = f.readlines()

            with open(file_path, "w", encoding="utf-8") as f:
                for line in modify_fn(file_contents):
                    f.write(line)
        except Exception as e:  # pylint: disable=broad-exception-caught
            logging.error("Error modifying %s: %s", file_path, e)


def object_param_api_ts(file_contents: list[str]) -> Generator[str, None, None]:
    """Modify the ObjectParamAPI.ts file."""
    for line in file_contents:
        if dedent(line).startswith("public api("):
            line = line.replace("public api(", "public api_(")
        yield line


def promise_api_ts(file_contents: list[str]) -> Generator[str, None, None]:
    """Modify the PromiseAPI.ts file."""
    for line in file_contents:
        if dedent(line).startswith("public api("):
            line = line.replace("public api(", "public api_(")
        yield line


def object_serializer_ts(file_contents: list[str]) -> Generator[str, None, None]:
    """Modify the ObjectSerializer.ts file."""
    for line in file_contents:
        if dedent(line).startswith('"Results": ResultsClass,'):
            line = ""  # lead to call of missing `getAttributeTypeMap` method
        yield line


if __name__ == "__main__":
    main()
