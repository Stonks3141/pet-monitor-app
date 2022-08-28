#!/bin/sh

git update-index --refresh
if [ $(git diff-index --quiet HEAD --) = 0 ]; then
  tag=$(git log -1 --pretty=%H)
else
  tag="latest"
fi

# [ -t 1 ] && echo "Building frontend container..."
# docker build ./client -t pet-monitor-app-client:test-$tag --target base
# [ -t 1 ] && echo "Testing frontend..."
# docker run --workdir /usr/local/src/pet-monitor-app pet-monitor-app-client:test-$tag "pnpm test"

[ -t 1 ] && echo "Building server container..."
docker build ./pet-monitor-app -t pet-monitor-app:test-$tag --target base
[ -t 1 ] && echo "Testing server..."
docker run --workdir /usr/local/src/pet-monitor-app pet-monitor-app:test-$tag "cargo test"

[ -t 1 ] && echo "Tests complete!"
