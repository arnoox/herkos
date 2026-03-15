//! General-purpose utility functions for code generation.

/// Build a call args vector by conditionally adding globals/memory/table/host.
pub fn build_inner_call_args(
    base_args: &[String],
    has_globals: bool,
    globals_expr: &str,
    has_memory: bool,
    memory_expr: &str,
    has_table: bool,
    table_expr: &str,
) -> Vec<String> {
    let mut call_args = base_args.to_vec();
    if has_globals {
        call_args.push(globals_expr.to_string());
    }
    if has_memory {
        call_args.push(memory_expr.to_string());
    }
    if has_table {
        call_args.push(table_expr.to_string());
    }
    call_args
}
