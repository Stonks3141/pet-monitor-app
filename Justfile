
default:
  @just --list

# Run checks for the workspace
check *ARGS:
  cargo fmt --check
  cargo clippy --all-features --workspace {{ARGS}} -- -D warnings

# Run `cargo test` with ARGS and print execution time
test *ARGS:
  DISABLE_VALIDATE_CONFIG=1 \
  cargo test --all-features --workspace {{ARGS}} -- -Z unstable-options --report-time

# Publish all crates in the workspace
publish TOKEN:
  cargo ws publish --yes --from-git --token {{TOKEN}}

# Build the program and install to PATH
install PATH='~/.local/bin/':
  cargo build --release
  mkdir -p {{PATH}}
  cp target/release/pet-monitor-app {{PATH}}
