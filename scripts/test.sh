#!/bin/sh

set -e

info() {
  if [ -t 1 ]; then
    echo -e "\e[32m$1\e[0m"
  else
    echo "$1"
  fi
}

git update-index --refresh
if [ $(git diff-index --quiet HEAD --) = 0 ]; then
  tag=$(git log -1 --pretty=%H)
else
  tag="latest"
fi

# info "Building frontend container..."
# docker build ./client -t pet-monitor-app-client:test-$tag --target base
# info "Testing frontend..."
# docker run --workdir /usr/local/src/pet-monitor-app pet-monitor-app-client:test-$tag "pnpm test"

info "Building server container..."
docker build ./pet-monitor-app -t pet-monitor-app:test-$tag --target base
info "Testing server..."
docker run \
--workdir /usr/local/src/pet-monitor-app \
--mount type=bind,src=$(pwd)/pet-monitor-app,dst=/usr/local/src/pet-monitor-app \
pet-monitor-app:test-$tag \
cargo test

info "Tests complete!"
