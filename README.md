# pet-monitor-app

This project is currently pre-alpha.
Meant to run on a Raspberry Pi. [v4l2](https://www.kernel.org/doc/html/v4.9/media/uapi/v4l/v4l2.html) is used for video.

- [pet-monitor-app](#pet-monitor-app)
  - [Installation](#installation)
    - [General](#general)
    - [Development](#development)
  - [Usage](#usage)
  - [Motivation](#motivation)
  - [Goals](#goals)
  - [Roadmap](#roadmap)
  - [Contributing](#contributing)
  - [Inspiration](#inspiration)

## Installation

### General

Docker coming soon.

### Development

Clone the repo and install npm/yarn, rustup, and gstreamer. Run `npm install` or `yarn add` in the `/client` directory. Run `cargo run` in the base directory to start

## Usage

In progress

## Motivation

I wanted to make a pet monitor without paying for one, so I used [fmp4streamer](https://github.com/soyersoyer/fmp4streamer). However, I was unsatisfied with the lack of security and features. This project aims to fix that, with support for TLS/HTTPS, JWT authentication, reverse proxy, and secure password storage. In the future, I hope to expand it with additional features, such as audio and video recording.

## Goals

- Secure
- Simple to install
- Featureful and attractive
- \>95% code coverage
- Complete documentation
- Streaming crate should have a powerful and high-level API

## Roadmap

- [ ] JSON web token authentication
- [ ] Secure password storage with `argon2`
- [ ] HTTPS
- [ ] Rust v4l2 streaming
- [x] Basic UI
- [ ] Docker container
- [ ] Binary distribution
- [x] Rewrite backend in Rust/Rocket
- [ ] GraphQL with Juniper
- [ ] Audio support
- [ ] Documentation
- [ ] As secure as possible without HTTPS
- [ ] Motion sensing to detect active periods
- [ ] Add activity overview to UI

## Contributing

PRs are welcome.

## Inspiration

This project was inspired by [soyersoyer/fmp4streamer](https://github.com/soyersoyer/fmp4streamer).
