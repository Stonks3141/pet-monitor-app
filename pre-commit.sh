#!/bin/sh

STASH_NAME="pre-commit-$(date +%s)"
git stash save --quiet --keep-index --include-untracked "$STASH_NAME"
echo "pre-commit"
cd client || exit 1
pnpm build || exit 1
pnpm test || exit 1
pnpm format || exit 1
pnpm lint || exit 1
cd ..
cargo check || exit 1
cargo check --release
cargo test
cargo fmt
cargo clippy

git stash pop --quiet

