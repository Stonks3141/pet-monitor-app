# syntax=docker/dockerfile:1

# build backend
# FROM rust:1.60-alpine AS build
# RUN apk add --no-cache v4l-utils-libs build-base
# WORKDIR /tmp/pet-monitor-app
# COPY . .
# RUN cargo build --release

FROM alpine:3.16
# COPY --from=build /tmp/pet-monitor-app/target/release/pet-monitor-app /usr/local/bin
COPY ../target/release/pet-monitor-app /usr/local/bin

ENTRYPOINT [ "pet-monitor-app" ]
EXPOSE 80
