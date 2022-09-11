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

info "Building server..."
docker build ./pet-monitor-app -t pet-monitor-app:$tag

info "Build complete! Run with \`docker run -it -p 80:80 -p 443:443 pet-monitor-app:$tag\`."
