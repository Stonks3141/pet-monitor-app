hash=$(git log -1 --pretty=%H)

echo "Building frontend bundle..."
docker build ./client -t Stonks3141/pet-monitor-app-client:$hash
echo "Copying bundle out of container..."
docker cp Stonks3141/pet-monitor-app-client:$hash:/usr/local/src/pet-monitor-app/dist ./client
echo "Building server..."
docker build ./pet-monitor-app -t Stonks3141/pet-monitor-app:$hash

echo "Build complete! You can find the binary at \
`./pet-monitor-app/target/release/pet-monitor-app`. \
Run with `docker run -it Stonks3141/pet-monitor-app:$hash`."