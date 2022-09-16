#!/bin/sh

set -e

info() {
  if [ -t 1 ]; then
    echo -e "\e[32m$1\e[0m"
  else
    echo "$1"
  fi
}

$(git update-index --refresh)
if [ $(git diff-index --quiet HEAD --) = 0 ]; then
  tag=$(git log -1 --pretty=%H)
else
  tag="latest"
fi

info "Building server container..."
docker build ./pet-monitor-app -t pet-monitor-app:test-$tag --target base
info "Checking formatting..."
docker run \
--mount type=bind,src=$(pwd)/pet-monitor-app,dst=/usr/local/src/pet-monitor-app \
pet-monitor-app:test-$tag \
cargo fmt --all -- --check
info "Linting server..."
docker run \
--mount type=bind,src=$(pwd)/pet-monitor-app,dst=/usr/local/src/pet-monitor-app \
pet-monitor-app:test-$tag \
cargo clippy

info "Linting complete!"
