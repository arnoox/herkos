# Pull Request

## Summary

Brief description of what this PR does (1-2 sentences).

## Motivation

Why is this change needed? What problem does it solve?

Closes #issue_number

## Changes

List of specific changes made:

- Added/modified feature X in `file.rs`
- Fixed bug Y that caused Z
- Updated documentation in `SPEC.md`

## Type of Change

- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Code refactoring (no functional changes)
- [ ] Test additions or updates

## Testing

### Test Coverage

- [ ] Unit tests added or updated
- [ ] Integration tests added or updated (in `herkos-tests/`)
- [ ] Kani proofs added or updated (if modifying runtime)
- [ ] Manual testing performed

### Test Results

```bash
# Paste output of cargo test (or relevant subset)
cargo test
```

## Code Quality Checklist

- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy --all-targets -- -D warnings`)
- [ ] Code formatted (`cargo fmt`)
- [ ] `no_std` compliance maintained (if modifying `herkos-runtime`)
- [ ] Documentation updated (rustdoc comments, SPECIFICATION.md, etc.)
- [ ] CHANGELOG.md updated (if user-facing change)
- [ ] No new `unsafe` blocks (or justified with `// SAFETY:` comments)
- [ ] No panics in production code paths

## Performance Impact

Does this change affect performance?

- [ ] No performance impact
- [ ] Performance improved
- [ ] Performance may be affected (explain below)

Details:

## Breaking Changes

Does this PR introduce breaking changes?

- [ ] No breaking changes
- [ ] Yes, breaking changes (describe below and update version accordingly)

Details:

## Documentation

- [ ] Public API changes documented with `///` rustdoc comments
- [ ] Examples added to doc comments where appropriate
- [ ] SPECIFICATION.md updated (if architectural change)
- [ ] PLAN.md updated (if affecting roadmap)

## Additional Notes

Any additional information reviewers should know:
- Design decisions made
- Tradeoffs considered
- Known limitations
- Follow-up work needed

## Screenshots (if applicable)

For UI changes or generated code examples.

---

## Reviewer Checklist

For maintainers reviewing this PR:

- [ ] Code follows project conventions (see CONTRIBUTING.md)
- [ ] Tests are adequate and pass
- [ ] Documentation is clear and complete
- [ ] No security concerns
- [ ] Performance is acceptable
- [ ] Breaking changes are justified and documented
