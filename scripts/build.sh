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
if git diff-index --quiet HEAD --; then
  tag=$(git log -1 --pretty=%H)
else
  tag="latest"
fi

info "Building frontend bundle..."
docker build ./client -t pet-monitor-app-client:$tag

info "Copying bundle out of container..."
id=$(docker container create pet-monitor-app-client:$tag)
docker cp $id:/usr/local/src/pet-monitor-app/dist ./pet-monitor-app
docker rm -v $id

arch=$(rustc -vV | sed -rn "s/^host: ([^-]*).*\$/\1/p")
target=$arch-unknown-linux-gnu
info "Building server..."
docker build --build-arg target=$target ./pet-monitor-app -t stonks3141/pet-monitor-app:$tag

push=0
for arg in "$@"; do
  if [ "$arg" = "--push" ]; then
    push=1
    break
  fi
done
if [ "$DOCKER_TOKEN" != "" ] && [ $push -eq 1 ]; then
  docker login -u stonks3141 -p $DOCKER_TOKEN
  docker push stonks3141/pet-monitor-app:$tag
fi

info "Build complete! Run with \`docker run -it -p 80:80 -p 443:443 stonks3141/pet-monitor-app:$tag\`."
