# pet-monitor-app

[![CI](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/ci.yml/badge.svg)](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/ci.yml)
[![license](https://img.shields.io/static/v1?label=License&message=MIT&color=blue)](https://opensource.org/licenses/MIT)
[![loc](https://tokei.rs/b1/github/Stonks3141/pet-monitor-app)](https://github.com/XAMPPRocky/tokei)

This project is a combination of several components: a browser client, a backend, and a command-line interface. It provides a locally hosted, authenticated, and configurable video streaming application.

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
move it into your `$PATH`. Run these commands to start the server:

```sh
pet-monitor-app configure --password MY_PASSWORD && pet-monitor-app start
```

This first sets the password with the `configure` subcommand, and then starts
the server. You can view the page at [http://localhost](http://localhost).
To reset your password, run

```sh
pet-monitor-app configure --password NEW_PASSWORD
```

For a full list of command-line options, run with the `--help` flag.

The configuration file is located at
`~/.config/pet-monitor-app/pet-monitor-app.toml`. To enable TLS, add this to
the config file:

```toml
[tls]
port = 443
cert = "path/to/cert.pem"
key = "path/to/key.key"
```

You can now view the page at [https://localhost](https://localhost).

### Docker

To run with Docker, clone the repository and run the build script.

```sh
sudo ./scripts/build.sh
```

You can now run the container with

```sh
docker run -it -p 80:80 -p 443:443 stonks3141/pet-monitor-app
```

## Development

This project uses shell scripts to manage development and CI. To develop without Docker, you will need
a [Rust toolchain](https://www.rust-lang.org/tools/install) and
[pnpm](https://pnpm.io/)/[node](https://nodejs.org/).

Clone the repo. Install [Docker Desktop](https://www.docker.com/get-started/) and run `sudo docker compose up`
in the base directory. View the development server at [http://localhost:5173](http://localhost:5173).
The frontend will reload automatically as you make changes, but you will need
to restart the backend container. To run the development server, run `pnpm dev` in the `client/`
directory. In another shell, run `cargo run -- --config ./pet-monitor-app.toml` in the `pet-monitor-app/`
directory. The current password set in `pet-monitor-app/pet-monitor-app.toml` is "123".

To build a binary, run `sudo ./scripts/build.sh` in the base directory. This will build the frontend
in a Docker container, copy out the static files, and build the backend. Alternatively, run these
commands:

```sh
cd client
pnpm build
cd ../pet-monitor-app
rm -rf dist
cp -r ../client/dist .
cargo build --release
```

## Motivation

I wanted to make a pet monitor without paying for one, so I used
[fmp4streamer](https://github.com/soyersoyer/fmp4streamer). However, I was
unsatisfied with the lack of security and features (It wasn't designed for this
anyway, not their fault). This project aims to fix that, with support for
TLS/HTTPS, secure authentication, reverse proxy and containerization, and
secure password storage with argon2. In the future, I hope to expand it with
additional features, such as audio support and video recording.

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
- [ ] Rust v4l2 (libcamera?) streaming
- [ ] Audio support
- [ ] Recording video/audio to view later
- [ ] Documentation
- [x] In-browser configuration/server management

## Contributing

PRs are welcome. Look at [Github issues](https://github.com/Stonks3141/pet-monitor-app/issues)
for some ideas. If you contribute code, try to add unit/prop/integration tests for
any new functionality.

## Testing

pet-monitor-app uses [proptest](https://crates.io/crates/proptest) as well as Rust's built-in
unit and integration testing framework. To run tests, clone the repo and run

```sh
sudo ./scripts/test.sh
```

or

```sh
cd pet-monitor-app && cargo test
```

## Inspiration

This project was inspired by [soyersoyer/fmp4streamer](https://github.com/soyersoyer/fmp4streamer).
