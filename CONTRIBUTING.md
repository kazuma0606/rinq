# Contributing to RINQ

Thank you for your interest in contributing to RINQ!

## AI-Assisted Development

RINQ is openly developed with AI assistance (Claude Code / Anthropic). We welcome contributions that are also AI-assisted — there is no requirement to disclose AI use, but if you used an AI tool you may optionally add `# AI-assisted` to your commit message. This project believes AI-assisted coding is a valid and productive approach to open-source development.

## Contribution Sizes

### Small (bug fixes, doc improvements, typo fixes)
- No prior discussion needed.
- Open a PR directly with a clear title and description.

### Medium (new operators, new test coverage, performance improvements)
- Open an issue first to describe what you want to add and why.
- Wait for a thumbs-up before starting significant work.

### Large (new crates, API-breaking changes, architectural changes)
- Open an issue and discuss the design thoroughly before writing any code.
- Large changes need consensus on approach before a PR is welcome.

## Pre-PR Checklist

Before opening a pull request, make sure:

```bash
# All tests pass
cargo test --workspace

# No Clippy warnings
cargo clippy --workspace --all-features -- -D warnings

# Formatted correctly
cargo fmt --all --check

# Doc tests pass
cargo test --doc
```

## The `versions/` Directory

The `versions/` directory contains internal AI coding specs, plans, and task lists used during development. Contributors do **not** need to read or update these files — they are internal tooling for AI-assisted development sessions.

## Pull Request Guidelines

- Keep PRs focused: one feature or fix per PR.
- Add tests for new functionality.
- Update `CHANGELOG.md` under `[Unreleased]` with a brief description of your change.
- Reference related issues with `Closes #N` or `Related to #N` in the PR description.

## Questions?

Open an issue with the `question` label — we're happy to help!
