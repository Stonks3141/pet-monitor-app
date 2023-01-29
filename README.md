# pet-monitor-app

[![CI](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/ci.yml/badge.svg)](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/ci.yml)
[![license](https://img.shields.io/static/v1?label=License&message=MIT&color=blue)](https://opensource.org/licenses/MIT)
[![loc](https://tokei.rs/b1/github/Stonks3141/pet-monitor-app)](https://github.com/XAMPPRocky/tokei)

pet-monitor-app is a simple video streaming server for Linux. It provides
out-of-the-box support for HTTPS and password authentication.

- [pet-monitor-app](#pet-monitor-app)
  - [Quickstart](#quickstart)
  - [Development](#development)
  - [Motivation](#motivation)
  - [Goals](#goals)
  - [Roadmap](#roadmap)
  - [Contributing](#contributing)
  - [Inspiration](#inspiration)

## Installation

### Precompiled Binary

Install libx264 using your system's package manager. Download the
binary and corresponding `.sha256` file for your OS/architecture from the
[releases](https://github.com/Stonks3141/pet-monitor-app/releases) page.
Run `sha256sum --check pet-monitor-app-VERSION-TARGET.sha256` to verify the
checksum. If it is correct, move the binary into `~/.local/bin`.

### Building from Source

Install [rustup][rustup] and run these commands:

```sh
git clone https://github.com/Stonks3141/pet-monitor-app.git
cd pet-monitor-app
cargo build --release
cp target/release/pet-monitor-app ~/.local/bin
```

If you have [just][just] installed, you can run `just install` after cloning.

## Usage

Run these commands to start the server:

```sh
pet-monitor-app set-password MY_PASSWORD
pet-monitor-app start
```

This first sets the password with the `set-password` subcommand, and then starts
the server. You can view the page at [http://localhost:8080](http://localhost:8080).
To reset your password, run the `set-password` subcommand again.

For a full list of command-line options, run with the `--help` flag.

The configuration file is located at `~/.config/pet-monitor-app/config.toml`.
To enable TLS, add this to the config file:

```toml
[tls]
port = 8443
cert = "path/to/cert.pem"
key = "path/to/key.key"
```

You can now view the page at [https://localhost:8443](https://localhost:8443).

Running pet-monitor-app as root is not necessary and should be avoided. If you
want your server to listen on port 80 or 443, you should set up NAT forwarding
to forward external port 80 to internal port 8080. If this is not possible,
install nginx and use it to reverse proxy port 80 or 443 to pet-monitor-app.

## Configuration

`~/.config/pet-monitor-app/config.toml`

```toml
# The argon2 hash of the password
password_hash = '$argon2id$v=19$m=32768,t=8,p=4$19nFC/J5TEtjGGePEsLX+g$KmofOFmpLIBwqC7PkpHYyQyTiQF82IoBKanci2Dn5Ds'
# The secret used to sign authentication tokens
jwt_secret = 'DkTeDKts0tinlvmfUtbnepKqYHeX1B8w7sQ5LG9KW+s='
# The timeout for auth tokens in seconds
jwt_timeout = 345600
# The domain to serve from
domain = 'localhost'
# The IP to listen on
host = '127.0.0.1'
# The port to listen on
port = 8080
# The device to capture video from
device = '/dev/video0'
# The format to capture video in
format = 'YUYV'
# The resolution to capture video in
resolution = [640, 480]
# The frame interval to use
# The framerate is equal to the first part divided by the second
interval = [1, 30]
# The video rotation (must be one of 0, 90, 180, or 270)
rotation = 0

# Additional V4L2 controls
[v4l2Controls]
foo = 0

# TLS configuration
[tls]
# The port to listen on for TLS
port = 8443
# Path to the SSL certificate
cert = "path/to/cert.pem"
# Path to the SSL certificate key
key = "path/to/key.key"
```

## Development

You will need to install [rustup][rustup] and [just][just].

To start the server, run `cargo run -- start`. In debug mode, the server will read
client files from disk, and in release mode, they will be bundled into the binary.

To build the program, run `cargo build --release`. The binary should be located
at `target/release/pet-monitor-app`.

## Contributing

PRs are welcome. If you contribute code, try to add integration tests for any new
functionality.

## Inspiration

This project was inspired by [soyersoyer/fmp4streamer](https://github.com/soyersoyer/fmp4streamer).

[rustup]: https://www.rust-lang.org/learn/get-started
[just]: https://github.com/casey/just
