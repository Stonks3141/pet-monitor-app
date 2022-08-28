#!/bin/sh

git update-index --refresh
if [ git diff-index --quiet HEAD -- ]; then
  tag=$(git log -1 --pretty=%H)
else
  tag="latest"
fi

echo "Building frontend bundle..."
docker build ./client -t pet-monitor-app-client:$tag
echo "Copying bundle out of container..."
docker cp pet-monitor-app-client:$tag:/usr/local/src/pet-monitor-app/dist ./client
echo "Building server..."
docker build ./pet-monitor-app -t pet-monitor-app:$tag

echo "Build complete! Run with \
`docker run -it pet-monitor-app:$tag`."
