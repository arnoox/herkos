#!/usr/bin/env python3
"""Generate {test} needs from herkos integration test files.

Scans crates/herkos-tests/tests/*.rs for #[test] functions and produces
MyST markdown with {test} directives. Links are read from test_links.toml.
"""

import re
import sys

if sys.version_info >= (3, 11):
    import tomllib
else:
    try:
        import tomllib
    except ImportError:
        import tomli as tomllib  # type: ignore[no-redef]

from pathlib import Path

SCRIPTS_DIR = Path(__file__).parent
REPO_ROOT = SCRIPTS_DIR.parent.parent
TESTS_DIR = REPO_ROOT / "crates" / "herkos-tests" / "tests"
LINKS_PATH = SCRIPTS_DIR / "test_links.toml"
OUTPUT_PATH = SCRIPTS_DIR.parent / "traceability" / "tests.md"

TEST_FN_RE = re.compile(r"fn\s+(test_\w+)")


def extract_test_fns(path: Path) -> list[str]:
    """Extract #[test] function names from a Rust test file."""
    text = path.read_text()
    return TEST_FN_RE.findall(text)


def make_test_id(file_stem: str, fn_name: str) -> str:
    """Create a need ID like TEST_ARITHMETIC_ADD_CORRECTNESS."""
    return f"TEST_{file_stem.upper()}_{fn_name.removeprefix('test_').upper()}"


def main():
    with open(LINKS_PATH, "rb") as f:
        links_data = tomllib.load(f)

    lines = [
        "# Test Cases",
        "",
        "Auto-generated from `crates/herkos-tests/tests/*.rs`.",
        "",
    ]

    test_files = sorted(TESTS_DIR.glob("*.rs"))
    total = 0

    for test_file in test_files:
        stem = test_file.stem
        fns = extract_test_fns(test_file)
        test_fns = [fn for fn in fns if fn.startswith("test_")]

        if not test_fns:
            continue

        rel_path = test_file.relative_to(REPO_ROOT)
        lines.append(f"## {stem}")
        lines.append("")

        # Get file-level verifies links
        file_links = links_data.get(stem, {})
        verifies = file_links.get("verifies", [])
        verifies_str = ", ".join(verifies) if verifies else ""

        for fn_name in test_fns:
            need_id = make_test_id(stem, fn_name)
            lines.append(f"```{{test}} {fn_name}")
            lines.append(f":id: {need_id}")
            lines.append(f":source_file: {rel_path}")
            lines.append(f":tags: {stem}")
            if verifies_str:
                lines.append(f":verifies: {verifies_str}")
            lines.append("")
            lines.append(f"`{fn_name}` in `{rel_path}`.")
            lines.append("```")
            lines.append("")
            total += 1

    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_PATH.write_text("\n".join(lines))
    print(f"Generated {OUTPUT_PATH} with {total} test needs")


if __name__ == "__main__":
    main()
