#!/usr/bin/env python3
"""Generate {impl} needs for herkos implementation modules.

Produces a MyST markdown file with {impl} directives for the key source
modules in herkos-core and herkos-runtime.
"""

from pathlib import Path

SCRIPTS_DIR = Path(__file__).parent
OUTPUT_PATH = SCRIPTS_DIR.parent / "traceability" / "impl.md"

# (need_id, title, source_file, satisfies_list, implements_list, tags)
IMPL_MODULES = [
    (
        "IMPL_PARSER",
        "Wasm binary parser",
        "crates/herkos-core/src/parser/mod.rs",
        ["REQ_TRANS_FUNCTIONS"],
        ["WASM_BIN_MODULES", "WASM_BIN_SECTIONS", "WASM_BIN_TYPES", "WASM_BIN_INSTRUCTIONS"],
        "parser, binary",
    ),
    (
        "IMPL_IR_BUILDER",
        "Wasm-to-IR translation",
        "crates/herkos-core/src/ir/builder/",
        ["REQ_TRANS_FUNCTIONS", "REQ_TRANS_CONTROL_FLOW"],
        ["WASM_MOD_FUNCTIONS", "WASM_EXEC_CONTROL", "WASM_EXEC_CALLS"],
        "ir, builder",
    ),
    (
        "IMPL_BACKEND_SAFE",
        "Safe backend (bounds-checked codegen)",
        "crates/herkos-core/src/backend/safe.rs",
        ["REQ_MEM_BOUNDS_CHECKED", "REQ_TRANS_SELF_CONTAINED"],
        ["WASM_EXEC_INTEGER_OPS", "WASM_EXEC_FLOAT_OPS", "WASM_EXEC_MEMORY"],
        "backend, safe, codegen",
    ),
    (
        "IMPL_CODEGEN_INSTR",
        "Instruction code generation",
        "crates/herkos-core/src/codegen/instruction.rs",
        ["REQ_TRANS_FUNCTIONS"],
        ["WASM_EXEC_INTEGER_OPS", "WASM_EXEC_FLOAT_OPS", "WASM_EXEC_CONVERSIONS",
         "WASM_EXEC_MEMORY", "WASM_EXEC_CALLS"],
        "codegen, instruction",
    ),
    (
        "IMPL_CODEGEN_MODULE",
        "Module code generation",
        "crates/herkos-core/src/codegen/module.rs",
        ["REQ_MOD_TWO_TYPES", "REQ_TRANS_SELF_CONTAINED", "REQ_TRANS_VERSION_INFO"],
        ["WASM_MOD_FUNCTIONS", "WASM_MOD_EXPORTS", "WASM_MOD_IMPORTS",
         "WASM_MOD_GLOBALS", "WASM_MOD_TABLES"],
        "codegen, module",
    ),
    (
        "IMPL_RUNTIME_MEMORY",
        "IsolatedMemory runtime",
        "crates/herkos-runtime/src/memory.rs",
        ["REQ_MEM_PAGE_MODEL", "REQ_MEM_COMPILE_TIME_SIZE", "REQ_MEM_BOUNDS_CHECKED",
         "REQ_MEM_GROW_NO_ALLOC", "REQ_MEM_BULK_OPS"],
        ["WASM_MEMORY_TYPE", "WASM_MOD_MEMORIES", "WASM_MEMORY_SIZE", "WASM_MEMORY_GROW",
         "WASM_EXEC_MEMORY"],
        "runtime, memory",
    ),
    (
        "IMPL_RUNTIME_TABLE",
        "Table runtime (indirect calls)",
        "crates/herkos-runtime/src/table.rs",
        ["REQ_MOD_TABLE", "REQ_TRANS_INDIRECT_CALLS"],
        ["WASM_TABLE_TYPE", "WASM_MOD_TABLES", "WASM_MOD_ELEM", "WASM_CALL_INDIRECT"],
        "runtime, table",
    ),
    (
        "IMPL_RUNTIME_OPS",
        "Wasm arithmetic operations",
        "crates/herkos-runtime/src/ops.rs",
        ["REQ_ERR_TRAPS"],
        ["WASM_EXEC_INTEGER_OPS", "WASM_EXEC_FLOAT_OPS", "WASM_EXEC_CONVERSIONS"],
        "runtime, ops, arithmetic",
    ),
]


def main():
    lines = [
        "# Implementation Modules",
        "",
        "Key implementation modules in herkos-core and herkos-runtime.",
        "",
    ]

    for need_id, title, source_file, satisfies, implements, tags in IMPL_MODULES:
        satisfies_str = ", ".join(satisfies)
        implements_str = ", ".join(implements)
        lines.append(f"```{{impl}} {title}")
        lines.append(f":id: {need_id}")
        lines.append(f":source_file: {source_file}")
        lines.append(f":tags: {tags}")
        lines.append(f":satisfies: {satisfies_str}")
        lines.append(f":implements: {implements_str}")
        lines.append("")
        lines.append(f"`{source_file}`")
        lines.append("```")
        lines.append("")

    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_PATH.write_text("\n".join(lines))
    print(f"Generated {OUTPUT_PATH} with {len(IMPL_MODULES)} impl needs")


if __name__ == "__main__":
    main()
