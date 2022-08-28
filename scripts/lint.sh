#!/bin/sh

git update-index --refresh
if [ $(git diff-index --quiet HEAD --) = 0 ]; then
  tag=$(git log -1 --pretty=%H)
else
  tag="latest"
fi

echo "Building server container..."
docker build ./pet-monitor-app -t pet-monitor-app:test-$tag --target base
echo "Linting server..."
docker run --workdir /usr/local/src/pet-monitor-app pet-monitor-app:test-$tag "cargo clippy"

echo "Lint complete!"
