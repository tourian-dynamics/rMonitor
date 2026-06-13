#!/bin/bash
# release.sh: Run tests, build, commit, tag, push, and create a GitHub release.
# Usage: ./scripts/release.sh <version> [--draft]
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

VERSION=""
DRAFT_FLAG=""

# Parse arguments
for arg in "$@"; do
    if [ "$arg" = "--draft" ]; then
        DRAFT_FLAG="--draft"
    elif [ -z "$VERSION" ]; then
        VERSION="$arg"
    fi
done

if [ -z "$VERSION" ]; then
    VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d '"' -f2)
fi

echo "=== Releasing pulse v$VERSION ==="

# 1. Run tests
"$SCRIPT_DIR/test.sh"

# 2. Build binaries and packages
"$SCRIPT_DIR/build.sh"

# 3. Git commit, tag and push
echo "=== Committing release artifacts ==="
git add -A

# Check if there are changes to commit
if ! git diff --cached --quiet; then
    git commit -m "release: pulse $VERSION"
else
    echo "No changes to commit."
fi

echo "=== Creating Git tag v$VERSION ==="
# Delete tag locally if it already exists to avoid conflict
git tag -d "v$VERSION" 2>/dev/null || true
git tag -a "v$VERSION" -m "pulse v$VERSION"

echo "=== Pushing to Git remote ==="
BRANCH=$(git branch --show-current)
git push origin "$BRANCH" --follow-tags

# 4. Create GitHub Release using gh CLI if available
if command -v gh &> /dev/null; then
    echo "=== Creating GitHub Release ==="
    ASSETS=()
    for file in "$PROJECT_ROOT/dist/binaries"/*; do
        if [ -f "$file" ]; then
            ASSETS+=("$file")
        fi
    done
    
    if [ ${#ASSETS[@]} -gt 0 ]; then
        gh_args=("release" "create" "v$VERSION" "${ASSETS[@]}" "--title" "pulse v$VERSION" "--generate-notes")
        if [ -n "$DRAFT_FLAG" ]; then
            gh_args+=("$DRAFT_FLAG")
        fi
        gh "${gh_args[@]}"
        echo "GitHub release created successfully."
    else
        echo "No assets found to upload."
    fi
else
    echo "Skipping GitHub release creation (gh CLI not installed)."
fi

echo "=== Release pulse v$VERSION completed successfully! ==="
