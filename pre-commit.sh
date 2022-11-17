#!/bin/sh

STASH_NAME="pre-commit-$(date +%s)"
git stash save --quiet --keep-index --include-untracked "$STASH_NAME"

cd client || exit
pnpm build
pnpm test
pnpm format
pnpm lint
cd ..
cargo check
cargo check --release
cargo test
cargo fmt
cargo clippy

git stash pop --quiet

