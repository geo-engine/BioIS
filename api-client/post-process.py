#!/usr/bin/env python3

"""
Post-processing of generated code.
"""

import logging
from collections.abc import Callable, Generator
from pathlib import Path
from textwrap import dedent
from typing import TypeAlias

FileModifier: TypeAlias = Callable[[list[str]], Generator[str, None, None]]
INDENT = "    "


def file_modifications() -> Generator[tuple[Path, FileModifier], None, None]:
    """Return a generator of file paths and their corresponding modification functions."""

    yield Path("models/all.ts"), all_ts
    yield Path("models/ObjectSerializer.ts"), object_serializer_ts
    yield Path("models/Results.ts"), results_ts
    yield Path("types/ObjectParamAPI.ts"), object_param_api_ts
    yield Path("types/PromiseAPI.ts"), promise_api_ts


def main():
    """Main function to perform file modifications."""

    logging.basicConfig(level=logging.INFO, format="[%(levelname)s] %(message)s")
    logger = logging.getLogger(__name__)

    subdir = Path("typescript")
    for file_path, modify_fn in file_modifications():
        full_path = subdir / file_path
        logger.info("Modifying %s…", full_path)

        try:
            # 1. Read file contents
            file_contents = full_path.read_text(encoding="utf-8").splitlines(
                keepends=True
            )

            # 2. Transform contents (might fail inside modify_fn)
            new_contents = modify_fn(file_contents)

            # 3. Write back only after transformation succeeds
            full_path.write_text("".join(new_contents), encoding="utf-8")

        except OSError as e:
            # Catches FileNotFoundError, PermissionError, IsADirectoryError, etc.
            logger.error("I/O error modifying %s: %s", full_path, e)

        except Exception:
            # Catches unexpected logic or parsing bugs in modify_fn
            # exc_info=True captures the full traceback in the log
            logger.exception("Unexpected error executing modify_fn on %s", full_path)


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


def results_ts(file_contents: list[str]) -> Generator[str, None, None]:
    """Modify the Results.ts file."""
    for line in file_contents:
        if dedent(line).startswith("import { HttpFile } from '../http/http';"):
            line = line + "import { InlineOrRefData } from './InlineOrRefData';\n"
        yield line


def object_serializer_ts(file_contents: list[str]) -> Generator[str, None, None]:
    """Modify the ObjectSerializer.ts file."""
    for line in file_contents:
        if dedent(line).startswith('"Results": ResultsClass,'):
            line = ""  # lead to call of missing `getAttributeTypeMap` method
        if dedent(line).startswith('"Input": InputClass,'):
            line = ""  # lead to call of missing `getAttributeTypeMap` method
        yield line


def all_ts(file_contents: list[str]) -> Generator[str, None, None]:
    """Modify all TypeScript files."""
    yield from file_contents

    # add to bottom
    yield "export * from '../models/ObjectSerializer';" + "\n"


if __name__ == "__main__":
    main()
