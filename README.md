# pet-monitor-app

[![Repository][repo]](https://github.com/Stonks3141/pet-monitor-app)
[![crates.io][cratesio]](https://crates.io/crates/pet-monitor-app)
[![CI][ci]](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/ci.yml)
[![License][license]](https://opensource.org/licenses/MIT)

This project is dedicated to [Nyx](https://github.com/nyxkrage), who was able to obtain a copy of the H.264 standard for me.

![PeepoHeart](https://cdn3.emoji.gg/emojis/2316-peepoheart.png)

pet-monitor-app is a simple video streaming server for Linux. It provides
out-of-the-box support for HTTPS and password authentication.

- [pet-monitor-app](#pet-monitor-app)
  - [Installation](#installation)
  - [Usage](#usage)
  - [Configuration](#configuration)
  - [Development](#development)
  - [Contributing](#contributing)
  - [Inspiration](#inspiration)

## Installation

### Precompiled Binary

Download the binary and corresponding `.sha256` file for your OS/architecture
from the [releases](https://github.com/Stonks3141/pet-monitor-app/releases) page.
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

To start the server, run `STATIC_RELOAD=1 cargo run -- start`. The env var tells the
server to read static assets from disk, they will be bundled into the binary otherwise.

To build the program, run `cargo build --release`. The binary should be located
at `target/release/pet-monitor-app`.

## Contributing

PRs are welcome. If you contribute code, try to add integration tests for any new
functionality.

## Inspiration

This project was inspired by [soyersoyer/fmp4streamer](https://github.com/soyersoyer/fmp4streamer).

[repo]: https://img.shields.io/badge/Github-Stonks3141/pet--monitor--app-red?style=for-the-badge&logo=github
[cratesio]: https://img.shields.io/crates/v/pet-monitor-app?style=for-the-badge
[ci]: https://img.shields.io/github/actions/workflow/status/Stonks3141/pet-monitor-app/ci.yml?style=for-the-badge
[license]: https://img.shields.io/badge/License-MIT-blue?style=for-the-badge
[rustup]: https://www.rust-lang.org/learn/get-started
[just]: https://github.com/casey/just
