# pet-monitor-app

This project is currently pre-alpha.
Meant to run on a Raspberry Pi. [v4l2](https://www.kernel.org/doc/html/v4.9/media/uapi/v4l/v4l2.html) is used for video.

- [pet-monitor-app](#pet-monitor-app)
  - [Roadmap](#roadmap)
  - [Inspiration](#inspiration)

## Roadmap

Current plan is to write a Node module in Rust for streaming/recording/audio/processing and use Neon for JS interop.

- [x] REST API and authentication
- [ ] Secure transmission with hashing
- [ ] HTTPS
- [ ] Rust v4l2 node module
- [ ] UI
- [ ] Docker container
- [ ] Binary distribution
- [ ] Audio support
- [ ] Documentation
- [ ] Fallback for no SSL certificate
- [ ] Motion sensing to detect active periods
- [ ] Add activity overview to UI

## Inspiration

This project was inspired by [soyersoyer/fmp4streamer](https://github.com/soyersoyer/fmp4streamer).
