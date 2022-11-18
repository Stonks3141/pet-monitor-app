# pet-monitor-app

[![CI](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/ci.yml/badge.svg)](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/ci.yml)
[![license](https://img.shields.io/static/v1?label=License&message=MIT&color=blue)](https://opensource.org/licenses/MIT)
[![loc](https://tokei.rs/b1/github/Stonks3141/pet-monitor-app)](https://github.com/XAMPPRocky/tokei)

pet-monitor-app is a simple video streaming server for Linux. It provides out-of-the-box support for HTTPS and password authentication.

- [pet-monitor-app](#pet-monitor-app)
  - [Quickstart](#quickstart)
  - [Development](#development)
  - [Motivation](#motivation)
  - [Goals](#goals)
  - [Roadmap](#roadmap)
  - [Contributing](#contributing)
  - [Inspiration](#inspiration)

## Quickstart

The only component needed to run pet-monitor-app is the binary. It handles
config file management, static file serving, and TLS/HTTPS. There is no need
for a reverse proxy.

Download the binary for your OS/architecture from the
[releases](https://github.com/Stonks3141/pet-monitor-app/releases) page and
move it into `~/.local/bin`. Run these commands to start the server:

```sh
pet-monitor-app configure --password MY_PASSWORD --regen-secret
sudo pet-monitor-app start
```

This first sets the password with the `configure` subcommand, and then starts
the server. Sudo is needed to listen on port 80. You can view the page at
[http://localhost](http://localhost). To reset your password, run

```sh
pet-monitor-app configure --password NEW_PASSWORD
```

For a full list of command-line options, run with the `--help` flag.

The configuration file is located at
`~/.config/pet-monitor-app/config.toml`. To enable TLS, add this to
the config file:

```toml
[tls]
port = 443
cert = "path/to/cert.pem"
key = "path/to/key.key"
```

You can now view the page at [https://localhost](https://localhost).

## Development

You will need to install [rustup](https://www.rust-lang.org/learn/get-started), [node](https://nodejs.org),
and [pnpm](https://pnpm.io/installation).

Clone the repository and set up the pre-commit hook with

```sh
git clone https://github.com/Stonks3141/pet-monitor-app.git
cd pet-monitor-app
ln -s pre-commit.sh .git/hooks/pre-commit
```

To install dependencies, run `pnpm install` in the `client/` directory. To
start the frontend development server, run `pnpm dev` in the `client/` directory.
While the frontend is running, you can run `cargo run -- start` in the
`pet-monitor-app/` directory to start the backend. Vite should proxy to the
backend automatically. The client bundle is not included in the binary unless
you build in release mode.

To build a binary, run these commands:

```sh
cd client
pnpm build
cd ../pet-monitor-app
rm -rf dist
cp -r ../client/build ./dist
cargo build --release
```

This builds the frontend bundle, copies it into the `pet-monitor-app/` directory, and builds the binary.
The binary should be located at `pet-monitor-app/target/release/pet-monitor-app`.

## Motivation

I wanted to have a pet monitor without buying one, so I used my Raspberry Pi Zero
and [fmp4streamer](https://github.com/soyersoyer/fmp4streamer). However, I was
unsatisfied with the lack of security and features (It wasn't designed for this
anyway, not their fault). This project aims to remedy that, with support for
TLS/HTTPS, secure authentication, and secure password storage with argon2. In the
future, I hope to expand it with additional features, such as audio support and
video recording.

## Goals

- Secure
- Simple to install/use/configure
- Locally hosted
- Featureful and attractive
- Tested
- Documented

## Roadmap

- [x] Basic UI
- [x] JSON web token authentication
- [x] Rewrite backend in Rust/Rocket
- [x] Secure password verification (argon2)
- [x] HTTPS
- [x] Proxy authenticated video requests to fmp4streamer as an intermediate solution
- [x] CLI and config file
- [x] Bundle static files into release binary
- [x] Rust v4l2 (libcamera?) streaming
- [ ] Audio support
- [ ] Recording video/audio to view later
- [x] Documentation
- [x] In-browser configuration/server management

## Contributing

PRs are welcome. Look at [Github issues](https://github.com/Stonks3141/pet-monitor-app/issues)
for some ideas. If you contribute code, try to add unit/property/integration tests for
any new functionality.

## Testing

pet-monitor-app uses [proptest](https://crates.io/crates/proptest),
[quickcheck](https://crates.io/crates/quickcheck), and Rust's built-in unit and integration
testing framework. To run tests, clone the repo and run

```sh
cd pet-monitor-app
cargo test
```

## Inspiration

This project was inspired by [soyersoyer/fmp4streamer](https://github.com/soyersoyer/fmp4streamer).
