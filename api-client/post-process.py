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
    yield Path("types/ObjectParamAPI.ts"), object_param_api_ts
    yield Path("types/PromiseAPI.ts"), promise_api_ts
    yield Path("apis/ProcessesApi.ts"), processes_api_ts


def main():
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
        except Exception as e:
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


def processes_api_ts(file_contents: list[str]) -> Generator[str, None, None]:
    """Modify the ProcessesAPI.ts file."""
    for line in file_contents:
        # TODO: fix in ogcapi's OpenAPI spec instead of patching the generated code
        if dedent(line).startswith('"{ [key: string]: InlineOrRefData; }", ""'):
            line = line.replace(
                '"{ [key: string]: InlineOrRefData; }", ""',
                '"any", ""',
            )
        yield line


if __name__ == "__main__":
    main()
