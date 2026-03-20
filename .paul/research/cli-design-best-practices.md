# Research: CLI Design Best Practices

**Date:** 2026-03-21
**Agent:** general-purpose (web)

---

## Summary

Comprehensive best practices for modern CLI tool design, drawn from aws-cli, gh, gcloud, kubectl, stripe-cli, and current ecosystem standards.

## 1. Configuration & Setup

### First-Run Experience
- Detect first run (no config), guide user through interactive setup
- Offer QuickStart (defaults) vs Advanced modes
- Collect: Atlassian site URL, auth method, default output format, default project
- Support `--non-interactive` for CI/CD
- Validate with test API call before storing

### Config File Locations

| Platform | Config | Cache |
|----------|--------|-------|
| Linux | `~/.config/shrug/` | `~/.cache/shrug/` |
| macOS | `~/.config/shrug/` | `~/.cache/shrug/` |
| Windows | `%APPDATA%\shrug\` | `%LOCALAPPDATA%\shrug\cache\` |

Override via `SHRUG_CONFIG_DIR` env var and `--config` flag.

### Config Precedence (highest → lowest)
1. Flags (`--site`, `--output json`)
2. Environment variables (`SHRUG_SITE`, `SHRUG_OUTPUT`)
3. Project-level config (`.shrug.toml` in cwd/git root)
4. User-level config (`~/.config/shrug/config.toml`)
5. System-wide config (`/etc/shrug/config.toml`)
6. Built-in defaults

### Format: TOML
Human-readable, comment support, type-safe, section-based. Rust ecosystem standard.

```toml
[defaults]
output = "table"
site = "mycompany"
profile = "work"

[profiles.work]
site = "mycompany.atlassian.net"
email = "me@company.com"

[profiles.personal]
site = "personal.atlassian.net"
email = "me@gmail.com"
```

## 2. Profile Management

Pattern from AWS CLI / kubectl / gcloud:
- **Profile = site + user + settings**
- `shrug profile use <name>` — set default
- `shrug --profile staging issues list` — per-command override
- `SHRUG_PROFILE=staging` — env override
- `shrug profile list`, `shrug profile show <name>`, `shrug profile create <name>`
- Credentials stored separately in OS keychain, referenced by profile name

## 3. Secret/Credential Management

### OS Keychain (never plaintext config)
| Platform | Backend |
|----------|---------|
| macOS | Keychain Access |
| Windows | Credential Manager |
| Linux | Secret Service (GNOME Keyring / KDE Wallet) |

### Auth Commands
- `shrug auth login` — interactive: choose method, enter creds, validate, store
- `shrug auth status` — current auth state
- `shrug auth logout` — remove from keychain
- `shrug auth refresh` — manual OAuth2 refresh
- **CI/CD env vars:** `SHRUG_API_TOKEN`, `SHRUG_EMAIL`, `SHRUG_SITE`
- **Never accept secrets via flags** (leak to ps/history)
- **Never log secrets**: auto-mask tokens in debug output

### Atlassian Auth Methods
1. API Token + Email (Basic Auth) — most common for personal use
2. OAuth 2.0 (3LO) — for apps/integrations, with auto-refresh
3. Personal Access Token — for Data Center/Server

## 4. Shell Completions

`shrug completion <shell>` — outputs script for bash, zsh, fish, PowerShell

**Hybrid approach:**
- **Static:** command names, subcommands, flag names (fast)
- **Dynamic:** project keys, issue keys, user names (fetched from API, cached 5 min)
- Fail silently when API unreachable

## 5. Output Formatting

| Format | When |
|--------|------|
| Table | Default for TTY (human) |
| JSON | Default for non-TTY (piping) |
| YAML | Complex nested data |
| CSV | Spreadsheet export |
| Plain | grep/awk friendly |

- **TTY detection:** table + colors for terminal, JSON + no color for pipes
- **Respect `NO_COLOR`** env var, `--color=auto|always|never`
- **Pager:** auto-pipe through `$PAGER` for long output, `--no-pager` to disable
- **Filtering:** `--fields` to select columns, `--query` for JMESPath

## 6. Help System

Progressive disclosure:
1. No args → brief help + pointer to `--help`
2. `--help` → full help with examples
3. `shrug help <topic>` → deep-dive
4. `--web` → opens browser to docs

Lead with examples, suggest corrections for typos.

## 7. Error Handling

Every error includes: what went wrong → why → what to do next.

### Exit Codes
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid usage |
| 3 | Auth failure |
| 4 | Not found (404) |
| 5 | Permission denied (403) |
| 10 | Rate limited (429) |
| 11 | Network error |
| 12 | Server error (5xx) |

### Rate Limit Handling
- Exponential backoff with jitter (2s → 4s → 8s → 16s, max 4 retries)
- Respect `Retry-After` header
- Monitor `X-RateLimit-NearLimit` proactively

## 8. Logging & Debugging

| Flag | Level | Shows |
|------|-------|-------|
| (default) | WARN+ | Errors and warnings only |
| `-v` | INFO | Progress, API endpoints |
| `-vv` | DEBUG | Full request/response headers |
| `--trace` | TRACE | Complete bodies (secrets masked) |

All debug output to **stderr**. Support `SHRUG_LOG_LEVEL` env var.

---
*Research completed: 2026-03-21*
