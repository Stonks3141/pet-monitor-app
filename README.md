# pet-monitor-app

[![CI](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/ci.yml/badge.svg)](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/ci.yml)
[![license](https://img.shields.io/static/v1?label=License&message=MIT&color=blue)](https://www.gnu.org/licenses/gpl-3.0.en.html)
[![loc](https://tokei.rs/github/Stonks3141/pet-monitor-app?category=code)](https://github.com/XAMPPRocky/tokei)

This project is currently pre-alpha.
Meant to run on a Raspberry Pi.
[v4l2](https://www.kernel.org/doc/html/v4.9/media/uapi/v4l/v4l2.html) will be
used for video, but I'm also considering libcamera since it's intended to
replace v4l2 for high-level usage.

- [pet-monitor-app](#pet-monitor-app)
  - [Quickstart](#quickstart)
    - [Docker](#docker)
  - [Development](#development)
  - [Motivation](#motivation)
  - [Goals](#goals)
  - [Roadmap](#roadmap)
  - [Contributing](#contributing)
  - [Inspiration](#inspiration)

## Quickstart

Download the binary for your OS/architecture from the
[releases](https://github.com/Stonks3141/pet-monitor-app/releases) page and
move it into your `$PATH`. Run this command to start the server:

```sh
pet-monitor-app --regen-secret --password MY_PASSWORD
```

The `regen-secret` flag generates a new secret for JWT signing, and the
`password` flag sets your password. You can view the page at
[http://localhost](http://localhost). On subsequent runs, these flags are not
necessary. To reset your password, run

```sh
pet-monitor-app --password NEW_PASSWORD
```

For a full list of command-line options, run with the `--help` flag.

The configuration file is located at
`~/.config/pet-monitor-app/pet-monitor-app.toml`. To enable TLS, add this to
the config file:

```toml
[tls]
cert = "path/to/cert.pem"
key = "path/to/key.key"
```

You can now view the page at [https://localhost](https://localhost).

### Docker

To run with Docker, clone the repository and run the build script.

```sh
sudo sh ./scripts/build.sh
```

You can now run the container with

```sh
docker run -it -p 80:80 -p 443:443 pet-monitor-app
```

## Development

Clone the repo. Install Docker and Docker Compose and run `sudo docker compose up`
in the base directory. View the development server at [http://localhost:5173](http://localhost:5173).
The frontend will reload automatically as you make changes, but you will need
to restart the backend container.

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

PRs are welcome. Look at Github issues for some ideas.

## Inspiration

This project was inspired by [soyersoyer/fmp4streamer](https://github.com/soyersoyer/fmp4streamer).
