# Research: Cross-Platform CLI Support (Windows/macOS/Linux)

**Date:** 2026-03-21
**Agent:** general-purpose (web)

---

## Summary

Rust is excellent for cross-platform CLIs. Key crates cover paths, credentials, terminals, completions, distribution, and testing. Single static binary per platform, no runtime dependencies.

## 1. File Paths

Use `directories` crate (v5) for platform-correct paths:
```rust
let dirs = ProjectDirs::from("com", "shrug", "shrug").unwrap();
dirs.config_dir()  // ~/.config/shrug | %APPDATA%\shrug | ~/Library/Application Support/shrug
dirs.cache_dir()   // ~/.cache/shrug | %LOCALAPPDATA%\shrug\cache | ~/Library/Caches/shrug
```

Always use `PathBuf`/`Path`, never string concatenation for paths.

## 2. Credential Storage

Use `keyring` crate (v3.6+) with explicit platform features:
```toml
[dependencies.keyring]
version = "3"
features = ["apple-native", "windows-native", "sync-secret-service"]
```

**Fallback** when no keyring available (containers, headless): encrypted file with `aes-gcm` + `argon2` key derivation from passphrase.

## 3. Terminal & Console

| Need | Crate | Notes |
|------|-------|-------|
| Colors | `owo-colors` | Lightweight, recommended |
| Windows ANSI init | `enable-ansi-support` | Call once at startup |
| Full terminal control | `crossterm` | Pure Rust, no ncurses |
| Unicode width | `unicode-width` | Correct column alignment |
| Terminal size | `terminal_size` | Cross-platform |
| Prompts/input | `dialoguer` | Password, confirm, select |
| Progress bars | `indicatif` | Spinners, progress |

- Respect `NO_COLOR`, `TERM=dumb`, `--color=auto|always|never`
- Default to 80 columns when detection fails

## 4. Shell Completions

Use `clap_complete` (official clap companion):
- Bash, Zsh, Fish, PowerShell, Elvish, Nushell
- `shrug completions <shell>` generates script
- Can also generate at build time via build.rs

## 5. Distribution & Installation

| Channel | Platform | Tool |
|---------|----------|------|
| `cargo install` | All | Source build |
| `cargo binstall` | All | Pre-built binary |
| Homebrew tap | macOS + Linux | `brew install` |
| Scoop | Windows | Manifest in GitHub |
| WinGet | Windows | Komac + WinGet Releaser |
| GitHub Releases | All | Pre-built archives |

### Release Automation: `cargo-dist`
- Builds on native CI runners (Linux/macOS/Windows)
- Creates GitHub Releases with platform archives
- Generates install scripts (shell + PowerShell)
- Integrates with `cargo-binstall`
- Supply-chain: GitHub Artifact Attestations + `cargo-auditable`

### Cross-Compilation
- `cross` (Docker-based) for Linux ARM targets
- Native CI runners for primary targets (most reliable)
- `x86_64-unknown-linux-musl` for fully static Linux binaries

### Code Signing
- macOS: `apple-codesign` (rcodesign) — works from any OS
- Windows: `cargo-codesign` wrapping `signtool.exe`

## 6. System Differences

| Area | Approach |
|------|----------|
| Signal handling | `ctrlc` crate (Ctrl+C cross-platform) |
| File locking | `fs2` crate (advisory locks) |
| Temp dirs | `tempfile` crate (auto-cleanup) |
| Env vars | `std::env::var()` — note Windows is case-insensitive |
| Proxy | `reqwest` handles HTTP_PROXY/HTTPS_PROXY automatically |
| TLS | Use `rustls` (not `native-tls`) — avoids 100ms cold start penalty |

## 7. Testing

GitHub Actions matrix: `ubuntu-latest`, `macos-latest`, `windows-latest`

| Crate | Purpose |
|-------|---------|
| `assert_cmd` | Run CLI binary, assert stdout/stderr/exit |
| `predicates` | Composable assertions |
| `insta` | Snapshot testing |
| `tempfile` | Temp files/dirs for test isolation |
| `assert_fs` | Filesystem assertions |

Use `#[cfg(target_os = "...")]` for platform-specific tests. Normalize path separators in snapshot tests.

## Recommended Cargo.toml

```toml
[dependencies]
clap = { version = "4", features = ["derive", "env"] }
clap_complete = "4"
directories = "5"
toml = "0.8"
serde = { version = "1", features = ["derive"] }
keyring = { version = "3", features = ["apple-native", "windows-native", "sync-secret-service"] }
owo-colors = "4"
enable-ansi-support = "0.2"
dialoguer = "0.11"
indicatif = "0.17"
terminal_size = "0.4"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
tokio = { version = "1", features = ["full"] }
ctrlc = "3"
tempfile = "3"
fs2 = "0.4"
anyhow = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = "0.3"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
insta = "1"
assert_fs = "1"
```

---
*Research completed: 2026-03-21*
