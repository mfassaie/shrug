# Contributing to shrug

Thank you for your interest in contributing. This guide covers how to build, test, and submit changes.

## Prerequisites

- Rust stable (latest recommended)
- On Windows: MSVC toolchain (`stable-x86_64-pc-windows-msvc`)
- Git

## Building

```sh
cargo build
```

## Testing

```sh
# Unit + integration tests
cargo test

# Smoke tests (requires shrug on PATH)
cargo test --test smoke

# E2E tests against live Atlassian Cloud (requires credentials)
SHRUG_E2E_SITE=yoursite.atlassian.net \
SHRUG_E2E_EMAIL=you@example.com \
SHRUG_E2E_TOKEN=your-api-token \
cargo test --test live -- --test-threads=1
```

## Code quality checks

```sh
cargo clippy -- -D warnings
cargo fmt --check
```

Both must pass before submitting a PR.

## Submitting changes

1. Fork the repository and create a branch from `main`
2. Make your changes, keeping commits focused
3. Add tests for new functionality
4. Ensure `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check` all pass
5. Open a pull request with a clear description of what changed and why

## Code style

- Follow `rustfmt` defaults (run `cargo fmt`)
- No `unsafe` code (enforced by `[lints.rust] unsafe_code = "forbid"`)
- Keep error messages actionable: what happened, why, what to do about it

## Reporting bugs

Use the [bug report template](https://github.com/mfassaie/shrug/issues/new?template=bug_report.yml) on GitHub Issues.

## Platform notes

The `rust-toolchain.toml` in the repo specifies the Windows MSVC toolchain. On Linux or macOS, you may need to override this:

```sh
rustup override set stable
```

Or simply delete `rust-toolchain.toml` — the project builds with any recent stable Rust.
