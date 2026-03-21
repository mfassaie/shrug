---
phase: 08-distribution-polish
plan: 01
subsystem: infra
tags: [cargo-dist, github-actions, release, homebrew, scoop, musl, cross-platform]

requires:
  - phase: 01-project-foundation
    provides: Cargo.toml package metadata, .github/workflows/ci.yml
provides:
  - Tag-triggered release workflow for 4 platform targets
  - Homebrew formula template for macOS/Linux
  - Scoop manifest template for Windows
affects: []

tech-stack:
  added: [cargo-dist 0.27.0 (CI only)]
  patterns: [matrix build strategy, artifact upload/download, tag-triggered releases]

key-files:
  created: [.github/workflows/release.yml, dist/homebrew/shrug.rb, dist/scoop/shrug.json]
  modified: [Cargo.toml]

key-decisions:
  - "Direct cargo build per target (not cargo-dist CLI) — simpler workflow, fewer moving parts"
  - "gh release create for publishing — uses built-in GitHub CLI, no extra tools"
  - "musl-tools for Linux static binary — audit finding, required for musl target"

patterns-established:
  - "Release workflow separate from CI workflow"
  - "Template manifests with VERSION/SHA256 placeholders filled at release time"

duration: ~5min
completed: 2026-03-21
---

# Phase 8 Plan 01: cargo-dist Release Pipeline, Homebrew Tap, and Scoop Manifest Summary

**Tag-triggered GitHub Actions release workflow building for 4 targets (Linux musl, macOS x86_64, macOS aarch64, Windows MSVC), plus Homebrew formula and Scoop manifest templates**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~5 min |
| Completed | 2026-03-21 |
| Tasks | 2 completed |
| Files modified | 4 |
| New tests | 0 (configuration only) |
| Total tests | 386 |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Release workflow triggers on version tags | Pass | `on: push: tags: ["v*.*.*"]` configured |
| AC-2: Cargo.toml has cargo-dist metadata | Pass | [workspace.metadata.dist] section with 4 targets |
| AC-3: Homebrew formula template correct | Pass | Architecture detection, URL templates, install/test sections |
| AC-4: Scoop manifest template correct | Pass | Valid JSON, autoupdate, architecture section |
| AC-5: Release workflow YAML valid | Pass | Proper syntax, matrix strategy, permissions block |

## Accomplishments

- Created `.github/workflows/release.yml` with matrix build strategy covering all 4 targets. Linux job installs musl-tools and adds the musl rustup target. Unix builds produce tar.gz, Windows produces zip. A separate release job collects all artifacts and creates a GitHub Release.
- Added `[workspace.metadata.dist]` section to Cargo.toml for cargo-dist compatibility.
- Created Homebrew formula at `dist/homebrew/shrug.rb` with `Hardware::CPU.arm?` detection for Apple Silicon vs Intel, plus Linux support.
- Created Scoop manifest at `dist/scoop/shrug.json` with autoupdate and checkver for automatic version tracking.

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `.github/workflows/release.yml` | Created | Tag-triggered release workflow with matrix builds |
| `dist/homebrew/shrug.rb` | Created | Homebrew formula template |
| `dist/scoop/shrug.json` | Created | Scoop manifest template |
| `Cargo.toml` | Modified | Added [workspace.metadata.dist] section |

## Decisions Made

None beyond plan. Executed as specified.

## Deviations from Plan

None. Plan executed exactly as written.

## Issues Encountered

None.

## Next Phase Readiness

**Ready:**
- Release pipeline ready for first `git tag v0.1.0 && git push --tags`
- Homebrew/Scoop templates ready for first release (fill VERSION/SHA256)

**Concerns:**
- No binary signing/notarisation (deferred, noted in audit)
- Templates need manual SHA256 update per release (standard)

**Blockers:**
- None

---
*Phase: 08-distribution-polish, Plan: 01*
*Completed: 2026-03-21*
