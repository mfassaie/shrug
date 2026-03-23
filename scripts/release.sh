#!/usr/bin/env bash
# Release script for shrug — bumps version, runs checks, creates git tag.
#
# Usage:
#   ./scripts/release.sh 0.6.0
#   ./scripts/release.sh 0.6.0 --dry-run
#
# What it does:
#   1. Validates the version argument (X.Y.Z format)
#   2. Updates Cargo.toml version
#   3. Runs cargo build to update Cargo.lock
#   4. Runs cargo test (full suite)
#   5. Runs cargo clippy -- -D warnings
#   6. Verifies shrug --version matches
#   7. Creates annotated git tag vX.Y.Z
#   8. Prints next steps (commit + push)
#
# Prerequisites:
#   - Clean working tree (no uncommitted changes outside version files)
#   - All tests passing

set -euo pipefail

VERSION="${1:-}"
DRY_RUN="${2:-}"

if [[ -z "$VERSION" ]]; then
    echo "Usage: ./scripts/release.sh <version> [--dry-run]"
    echo "  e.g. ./scripts/release.sh 0.6.0"
    exit 1
fi

# Validate version format
if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "Error: Version must be in X.Y.Z format, got: $VERSION"
    exit 1
fi

echo "=== shrug release: v$VERSION ==="
echo ""

# Check for uncommitted source changes (allow .paul/ and Cargo files)
if git diff --name-only HEAD | grep -qvE '^(\.paul/|Cargo\.(toml|lock)$)'; then
    echo "Warning: uncommitted source changes detected."
    git diff --name-only HEAD | grep -vE '^(\.paul/|Cargo\.(toml|lock)$)'
    echo ""
    read -p "Continue anyway? [y/N] " confirm
    if [[ "$confirm" != "y" && "$confirm" != "Y" ]]; then
        echo "Aborted."
        exit 1
    fi
fi

# Step 1: Update Cargo.toml version
CURRENT=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo "[1/7] Bumping version: $CURRENT -> $VERSION"
if [[ "$DRY_RUN" == "--dry-run" ]]; then
    echo "  (dry-run: skipping)"
else
    sed -i "0,/^version = \"$CURRENT\"/s//version = \"$VERSION\"/" Cargo.toml
fi

# Step 2: Build to update Cargo.lock
echo "[2/7] Building..."
if [[ "$DRY_RUN" == "--dry-run" ]]; then
    echo "  (dry-run: skipping)"
else
    cargo build --quiet
fi

# Step 3: Run tests
echo "[3/7] Running tests..."
if [[ "$DRY_RUN" == "--dry-run" ]]; then
    echo "  (dry-run: skipping)"
else
    cargo test --quiet
fi

# Step 4: Clippy
echo "[4/7] Running clippy..."
if [[ "$DRY_RUN" == "--dry-run" ]]; then
    echo "  (dry-run: skipping)"
else
    cargo clippy -- -D warnings --quiet 2>/dev/null || cargo clippy -- -D warnings
fi

# Step 5: Verify version output
echo "[5/7] Verifying version..."
if [[ "$DRY_RUN" == "--dry-run" ]]; then
    echo "  (dry-run: would check 'shrug $VERSION')"
else
    ACTUAL=$(cargo run --quiet -- --version 2>/dev/null)
    EXPECTED="shrug $VERSION"
    if [[ "$ACTUAL" != "$EXPECTED" ]]; then
        echo "Error: version mismatch."
        echo "  Expected: $EXPECTED"
        echo "  Got:      $ACTUAL"
        exit 1
    fi
    echo "  Confirmed: $ACTUAL"
fi

# Step 6: Update .paul/PROJECT.md version
echo "[6/7] Updating .paul/PROJECT.md..."
if [[ "$DRY_RUN" == "--dry-run" ]]; then
    echo "  (dry-run: skipping)"
else
    if [[ -f .paul/PROJECT.md ]]; then
        sed -i "s/| Version | .* |/| Version | $VERSION |/" .paul/PROJECT.md
    fi
fi

# Step 7: Create git tag
echo "[7/7] Creating git tag v$VERSION..."
if [[ "$DRY_RUN" == "--dry-run" ]]; then
    echo "  (dry-run: would create tag v$VERSION)"
else
    if git tag -l "v$VERSION" | grep -q "v$VERSION"; then
        echo "  Warning: tag v$VERSION already exists. Skipping."
    else
        git tag -a "v$VERSION" -m "v$VERSION release"
        echo "  Created: v$VERSION"
    fi
fi

echo ""
echo "=== Release v$VERSION prepared ==="
echo ""
echo "Next steps:"
echo "  1. Review changes:  git diff"
echo "  2. Stage and commit: git add Cargo.toml Cargo.lock .paul/"
echo "     git commit -m 'release: v$VERSION'"
echo "  3. Push:            git push && git push origin v$VERSION"
