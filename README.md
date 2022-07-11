# pet-monitor-app

[![build](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/ci.yml/badge.svg)](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/ci.yml)
[![test](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/test.yml/badge.svg)](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/test.yml)
[![lint](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/lint.yml/badge.svg?style=flat-square)](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/lint.yml)
[![license](https://img.shields.io/static/v1?label=License&message=GPLv3&color=blue&style=flat-square)](https://www.gnu.org/licenses/gpl-3.0.en.html)
[![loc](https://img.shields.io/tokei/lines/github/Stonks3141/pet-monitor-app?style=flat-square)](https://github.com/XAMPPRocky/tokei)

This project is currently pre-alpha.
Meant to run on a Raspberry Pi.
[v4l2](https://www.kernel.org/doc/html/v4.9/media/uapi/v4l/v4l2.html) will be
used for video, but I'm also considering libcamera since it's intended to
replace v4l2 for high-level usage.

- [pet-monitor-app](#pet-monitor-app)
  - [Installation](#installation)
  - [Development](#development)
  - [Usage](#usage)
  - [Testing](#testing)
  - [Motivation](#motivation)
  - [Goals](#goals)
  - [Roadmap](#roadmap)
  - [Contributing](#contributing)
  - [Inspiration](#inspiration)

## Quickstart

Install Nginx and copy `nginx.conf` from the [releases] page into `/etc/nginx/`.
Download the static files from the [releases] page and extract the archive into
the `/etc/nginx/html/` directory. Download the correct binary for your
architecture and put it in `/usr/local/bin`. To start the server, run

```bash
$ nginx -s start
$ pet-monitor-app
```

You can use systemd to run the servers automatically and handle logs.

[releases]: https://Stonks3141/pet-monitor-app/releases/

## Development

Clone the repo and install [Node](https://nodejs.org),
[yarn](https://yarnpkg.com/getting-started/install) and
[rustup](https://www.rust-lang.org/learn/get-started). Run `yarn start` in the
`client/` directory to start the frontend dev server. Run `cargo run` in the
base directory to start the development server. This will try to serve files
from `client/build/`, so run `yarn build` first.

## Usage

Run `cargo run`. Set the `PASSWORD` env var the first time and any time you
want to change the password.

## Testing

Tests are a work in progress.

Test the server with `cargo test`, and the client with `yarn test` in the
`client/` directory.

## Motivation

I wanted to make a pet monitor without paying for one, so I used
[fmp4streamer](https://github.com/soyersoyer/fmp4streamer). However, I was
unsatisfied with the lack of security and features (It wasn't designed for this
anyway). This project aims to fix that, with support for TLS/HTTPS, secure
authentication, reverse proxy and containerization, and secure password storage
with argon2. In the future, I hope to expand it with additional features, such
as audio and video recording.

## Goals

- Secure
- Simple to install/use/configure
- Featureful and attractive
- Tested
- Documented

## Roadmap

- [x] Basic UI
- [x] JSON web token authentication
- [x] Rewrite backend in Rust/Rocket
- [x] Secure password verification (argon2)
- [x] Docker container
- [x] HTTPS (part of Docker, handled with Nginx reverse proxy)
- [ ] Proxy authenticated video requests to fmp4streamer as an intermediate solution
- [ ] Rust v4l2 (libcamera?) streaming
- [ ] Audio support
- [ ] Recording video/audio to view later
- [ ] Documentation
- [ ] In-browser configuration/server management

## Contributing

PRs are welcome. Look at Github issues for some ideas.

## Inspiration

This project was inspired by [soyersoyer/fmp4streamer](https://github.com/soyersoyer/fmp4streamer).
