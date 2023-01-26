
# Run `cargo run -- ARGS` and start the frontend dev server
run *ARGS:
  (trap 'kill 0' SIGINT; cargo run -- {{ARGS}} & just dev)

# Run `cargo test` with ARGS and print execution time
test *ARGS:
  cargo test --all-features --workspace {{ARGS}} -- -Z unstable-options --report-time

# Start the frontend dev server
dev:
  cd client && pnpm dev

# Build the frontend and binary
build:
  cd client && pnpm build
  rm -rf pet-monitor-app/build/
  cp -r client/build/ pet-monitor-app/
  cargo build --release
