#!/usr/bin/env python3
"""Generate wasm_spec needs for numeric instructions from wasm_1_0_instructions.toml.

Reads the curated TOML data file (derived from the W3C WebAssembly Core
Specification 1.0) and produces MyST markdown with {wasm_spec} directives.
"""

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
TOML_PATH = SCRIPTS_DIR / "wasm_1_0_instructions.toml"
OUTPUT_PATH = SCRIPTS_DIR.parent / "traceability" / "wasm_spec" / "instructions_numeric.md"


def make_id(prefix: str, op: str) -> str:
    """Convert e.g. ('i32', 'div_s') to 'WASM_I32_DIV_S'."""
    return f"WASM_{prefix.upper()}_{op.upper()}"


def make_opcode(prefix: str, op: str) -> str:
    """Convert e.g. ('i32', 'div_s') to 'i32.div_s'."""
    return f"{prefix}.{op}"


def generate_grouped_section(prefix: str, ops: list[str], section: str,
                             tags: list[str]) -> list[str]:
    """Generate {wasm_spec} directives for a group of same-prefix instructions."""
    lines = []
    for op in ops:
        need_id = make_id(prefix, op)
        opcode = make_opcode(prefix, op)
        tag_str = ", ".join(tags)
        lines.append(f"```{{wasm_spec}} {opcode}")
        lines.append(f":id: {need_id}")
        lines.append(f":wasm_section: {section}")
        lines.append(f":wasm_opcode: {opcode}")
        lines.append(f":tags: {tag_str}")
        lines.append("")
        lines.append(f"Wasm 1.0: `{opcode}` instruction.")
        lines.append("```")
        lines.append("")
    return lines


def generate_conversions(data: dict) -> list[str]:
    """Generate {wasm_spec} directives for conversion instructions."""
    lines = []
    section = data["section"]
    base_tags = data["tags"]
    for entry in data["ops"]:
        name = entry["name"]
        need_id = f"WASM_{entry['id_suffix']}"
        desc = entry["desc"]
        tag_str = ", ".join(base_tags)
        lines.append(f"```{{wasm_spec}} {name}")
        lines.append(f":id: {need_id}")
        lines.append(f":wasm_section: {section}")
        lines.append(f":wasm_opcode: {name}")
        lines.append(f":tags: {tag_str}")
        lines.append("")
        lines.append(f"Wasm 1.0: `{name}` — {desc}.")
        lines.append("```")
        lines.append("")
    return lines


def main():
    with open(TOML_PATH, "rb") as f:
        data = tomllib.load(f)

    lines = [
        "# Numeric Instructions",
        "",
        "Wasm 1.0 numeric instructions (§2.4.1): constants, unary, binary,",
        "test, comparison, and conversion operations.",
        "",
        "Source: [W3C WebAssembly Core Specification 1.0, §2.4.1]"
        "(https://www.w3.org/TR/wasm-core-1/#numeric-instructions%E2%91%A0)",
        "",
    ]

    # Constants
    lines.append("## Constants")
    lines.append("")
    for type_key in ["i32", "i64", "f32", "f64"]:
        const_data = data["constants"][type_key]
        tags = const_data["tags"]
        section = data["constants"]["section"]
        for op in const_data["ops"]:
            lines.extend(generate_grouped_section(type_key, [op], section, tags))

    # Per-type instruction groups
    for type_key in ["i32", "i64"]:
        lines.append(f"## {type_key} Instructions")
        lines.append("")

        for group_key in [f"{type_key}_unary", f"{type_key}_test",
                          f"{type_key}_binop", f"{type_key}_compare"]:
            group = data[group_key]
            lines.extend(generate_grouped_section(
                group["prefix"], group["ops"], group["section"], group["tags"]
            ))

    for type_key in ["f32", "f64"]:
        lines.append(f"## {type_key} Instructions")
        lines.append("")

        for group_key in [f"{type_key}_unary", f"{type_key}_binop",
                          f"{type_key}_compare"]:
            group = data[group_key]
            lines.extend(generate_grouped_section(
                group["prefix"], group["ops"], group["section"], group["tags"]
            ))

    # Conversions
    lines.append("## Conversions")
    lines.append("")
    lines.extend(generate_conversions(data["conversions"]))

    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_PATH.write_text("\n".join(lines))
    print(f"Generated {OUTPUT_PATH} with numeric instruction needs")


if __name__ == "__main__":
    main()
