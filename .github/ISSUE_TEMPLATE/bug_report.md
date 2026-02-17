---
name: Bug Report
about: Report a bug or unexpected behavior
title: "[BUG] "
labels: bug
assignees: ''
---

## Bug Description

A clear and concise description of what the bug is.

## Steps to Reproduce

1. Provide the WebAssembly input (attach `.wasm` or `.wat` file, or provide minimal example)
2. Command used to run herkos:
   ```bash
   cargo run -p herkos -- input.wasm --mode safe --output output.rs
   ```
3. Describe what happened

## Expected Behavior

What you expected to happen.

## Actual Behavior

What actually happened. Include error messages, stack traces, or incorrect generated code.

```
Paste error messages or unexpected output here
```

## Environment

- **herkos version**: (e.g., 0.1.0 or commit hash)
- **Rust version**: `rustc --version`
- **Operating System**: (e.g., Linux, macOS, Windows)
- **Which crate**: (herkos, herkos-runtime, or herkos-tests)

## Generated Code (if applicable)

If the bug involves incorrect code generation, please provide:

<details>
<summary>Generated Rust code</summary>

```rust
// Paste generated code here
```

</details>

## Additional Context

Add any other context about the problem here:
- Is this a regression? (did it work in a previous version?)
- WebAssembly module characteristics (size, features used, etc.)
- Relevant excerpts from SPECIFICATION.md if applicable

## Possible Fix

If you have a suggestion for how to fix the bug, describe it here.
