# Contributing to herkos

Thank you for your interest in contributing to herkos! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Code Quality](#code-quality)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)
- [Reporting Issues](#reporting-issues)

## Code of Conduct

This project adheres to a Code of Conduct. By participating, you are expected to uphold this code. Please read [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for details.

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Git
- Python 3.12+ (for documentation builds)
- uv (Python package manager, optional for docs)

### Building the Project

```bash
# Clone the repository
git clone https://github.com/YOUR_ORG/herkos.git
cd herkos

# Build all crates
cargo build

# Run tests
cargo test
```

### Repository Structure

The project is organized as a Rust workspace with three core crates:

- `crates/herkos/` — CLI transpiler
- `crates/herkos-runtime/` — `#![no_std]` runtime library

See [README.md](README.md) and [SPECIFICATION.md](SPECIFICATION.md) for architectural details.

## Development Workflow

1. **Fork the repository** to your GitHub account
2. **Clone your fork** locally
3. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```
4. **Make your changes** following the coding conventions
5. **Test thoroughly** (see [Testing](#testing) section)
6. **Commit your changes** with clear messages (see [Commit Messages](#commit-messages))
7. **Push to your fork** and submit a pull request

## Testing

All code changes must include appropriate tests.

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p herkos
cargo test -p herkos-runtime
cargo test -p herkos-tests

# Run Kani formal verification proofs
cargo kani --tests -p herkos-runtime
```

### Test Requirements

- **Unit tests**: Add tests for new functions/methods in `#[cfg(test)] mod tests`
- **Integration tests**: Add E2E tests in `crates/herkos-tests/tests/` for new features
- **Formal proofs**: Update Kani harnesses if modifying runtime memory operations
- **All tests must pass**: CI will reject PRs with failing tests

### Writing Tests

Example unit test:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feature() {
        // Arrange
        let input = /* ... */;

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

## Code Quality

All code must pass quality checks before merging:

### Linting

```bash
# Run Clippy with deny warnings
cargo clippy --all-targets -- -D warnings
```

Fix all Clippy warnings. Do not use `#[allow(clippy::...)]` without justification.

### Formatting

```bash
# Check formatting
cargo fmt --check

# Auto-format code
cargo fmt
```

All code must be formatted with `rustfmt` using the project's default settings.

### Documentation

- Add `///` doc comments to all public APIs
- Include examples in doc comments where appropriate
- Update [SPECIFICATION.md](SPECIFICATION.md) for architectural changes
- Update [PLAN.md](PLAN.md) for roadmap changes

### `no_std` Compliance

The `herkos-runtime` crate **must** remain `#![no_std]` compatible:

```bash
# Verify no_std build
cargo build -p herkos-runtime --no-default-features
```

- No `std::` imports in runtime code
- No panics in safe execution paths (use `Result<T, WasmTrap>`)
- No heap allocation without the `alloc` feature

### Safety and `unsafe` Code

- Avoid `unsafe` blocks unless absolutely necessary
- Every `unsafe` block requires a `// SAFETY:` comment explaining invariants
- Verified backend `unsafe` code requires `// PROOF:` comments referencing verification metadata

## Commit Messages

Follow these conventions for commit messages:

### Format

```
<type>: <short summary> (max 72 chars)

<optional detailed description>

<optional footer>
```

### Types

- `feat:` — New feature
- `fix:` — Bug fix
- `refactor:` — Code refactoring (no behavior change)
- `perf:` — Performance improvement
- `test:` — Adding or updating tests
- `docs:` — Documentation changes
- `chore:` — Maintenance tasks (dependencies, CI, etc.)
- `style:` — Formatting, missing semicolons, etc. (no code change)

### Examples

Good commit messages:
```
feat: add support for multi-value blocks in IR builder

Implements multi-value block support as specified in SPECIFICATION.md §5.
Adds tracking for block result types and proper stack management.

Closes #42
```

```
fix: correct i32 shift amount masking to match Wasm spec

i32 shift operations now mask shift amount to 5 bits (& 31) to match
WebAssembly specification behavior.
```

Bad commit messages:
```
update code        # Too vague
Fixed bug          # No detail
WIP                # Should not be committed
```

## Pull Request Process

### Before Submitting

1. Ensure all tests pass locally:
   ```bash
   cargo test
   cargo clippy --all-targets -- -D warnings
   cargo fmt --check
   ```

2. Update documentation if needed
3. Rebase on latest `main` if behind
4. Squash WIP/fixup commits into logical commits

### PR Checklist

- [ ] Tests added/updated for changes
- [ ] All tests pass (`cargo test`)
- [ ] No Clippy warnings (`cargo clippy`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Documentation updated if needed
- [ ] CHANGELOG.md updated (if user-facing change)
- [ ] Commit messages follow conventions
- [ ] PR description explains the change

### PR Description Template

```markdown
## Summary
Brief description of what this PR does.

## Motivation
Why is this change needed? What problem does it solve?

## Changes
- List of specific changes made
- File paths affected
- New features added or bugs fixed

## Testing
- [ ] Unit tests added
- [ ] Integration tests added
- [ ] Manual testing performed

## Related Issues
Closes #issue_number
```

### Review Process

1. A maintainer will review your PR
2. Address any requested changes by pushing new commits
3. Once approved, a maintainer will merge your PR
4. Your contribution will be acknowledged in CHANGELOG.md

## Reporting Issues

### Bug Reports

Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md):

- Describe the bug clearly
- Provide steps to reproduce
- Include expected vs. actual behavior
- Provide version information (`cargo --version`, `rustc --version`)
- Include relevant error messages or logs

### Feature Requests

Use the [feature request template](.github/ISSUE_TEMPLATE/feature_request.md):

- Describe the feature and its use case
- Explain why it would be valuable
- Suggest a possible implementation (optional)
- Reference relevant SPECIFICATION.md sections if applicable

### Security Issues

**Do NOT open public issues for security vulnerabilities.**

See [SECURITY.md](SECURITY.md) for responsible disclosure instructions.

## Development Guidelines

### Error Handling

- **Runtime library** (`herkos-runtime`): Use `Result<T, WasmTrap>` for errors
- **Transpiler library**: Use `anyhow::Result<T>` for transpilation errors
- **CLI binaries**: Use `anyhow` for user-facing error messages
- Never `panic!`, `unwrap()`, or `expect()` in production code paths

### Naming Conventions

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `snake_case` for functions, variables, modules
- Use `CamelCase` for types, traits
- Map Wasm spec terminology to Rust: `i32.load` → `i32_load`

### Performance Considerations

- Use the **outline pattern** for generic functions to prevent monomorphization bloat
- Profile before optimizing (use `cargo bench` for microbenchmarks)
- Document any performance-critical code sections
- See SPECIFICATION.md §13.3 for monomorphization mitigation strategies

## Questions?

- Open a [discussion](https://github.com/YOUR_ORG/herkos/discussions) for questions
- Read the [SPECIFICATION.md](SPECIFICATION.md) for technical details
- Check existing issues and PRs for similar topics

Thank you for contributing to herkos!
