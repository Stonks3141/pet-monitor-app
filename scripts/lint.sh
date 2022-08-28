#!/bin/sh

git update-index --refresh
if [ git diff-index --quiet HEAD -- ]; then
  tag=$(git log -1 --pretty=%H)
else
  tag="latest"
fi

if [ -t 1 ]; then; echo "Building server container..."; fi
docker build ./pet-monitor-app -t pet-monitor-app:test-$tag --target base
if [ -t 1 ]; then; echo "Linting server..."; fi
docker run --workdir /usr/local/src/pet-monitor-app pet-monitor-app:test-$tag "cargo clippy"

if [ -t 1 ]; then; echo "Lint complete!"; fi
