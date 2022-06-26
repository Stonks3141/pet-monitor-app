# pet-monitor-app

[![Build and test](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/ci.yml/badge.svg)](https://github.com/Stonks3141/pet-monitor-app/actions/workflows/ci.yml)

This project is currently pre-alpha.
Meant to run on a Raspberry Pi. [v4l2](https://www.kernel.org/doc/html/v4.9/media/uapi/v4l/v4l2.html) will be used for video, but I'm also considering libcamera since it's intended to replace v4l2 for high-level usage.

- [pet-monitor-app](#pet-monitor-app)
  - [Installation](#installation)
    - [General](#general)
    - [Development](#development)
  - [Usage](#usage)
  - [Testing](#testing)
  - [Motivation](#motivation)
  - [Goals](#goals)
  - [Roadmap](#roadmap)
  - [Contributing](#contributing)
  - [Inspiration](#inspiration)

## Installation

### General

Docker container coming soon.

### Development

Clone the repo and install node, yarn and rustup. Run `yarn start` in the `client/` directory to start the frontend dev server. Run `cargo run` in the base directory to start the development server. This will try to serve files from `client/build/`, so run `yarn build` first.

## Usage

Run `cargo run` with the SECRET env var set to a base64 secret and the PASSWORD var set to the password you want to use.

## Testing

Tests are a work in progress.

Test the server with `cargo test` in the root directory, and the client with `yarn run test` in the `client/` directory.

## Motivation

I wanted to make a pet monitor without paying for one, so I used [fmp4streamer](https://github.com/soyersoyer/fmp4streamer). However, I was unsatisfied with the lack of security and features (It wasn't designed for this anyway). This project aims to fix that, with support for TLS/HTTPS, JWT authentication and authorization, reverse proxy and containerization, and secure password storage. In the future, I hope to expand it with additional features, such as audio and video recording.

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
- [ ] As secure as possible without HTTPS
- [ ] GraphQL with Juniper
- [ ] In-browser configuration/server management

## Contributing

PRs are welcome. Look at Github issues for some ideas.

## Inspiration

This project was inspired by [soyersoyer/fmp4streamer](https://github.com/soyersoyer/fmp4streamer).