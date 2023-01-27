
default:
  @just --list

# Run `cargo run -- ARGS` and start the frontend dev server
run *ARGS:
  (trap 'kill 0' SIGINT; cargo run -- {{ARGS}} & just dev)

# Run checks for the workspace
check *ARGS:
  cargo fmt --check
  cargo clippy --all-features --workspace {{ARGS}} -- -D warnings

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

# Build the program and install to PATH
install PATH='~/.local/bin/': build
  mkdir -p {{PATH}}
  cp target/release/pet-monitor-app {{PATH}}
