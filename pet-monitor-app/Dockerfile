# syntax=docker/dockerfile:1

# Alpine uses musl instead of glibc
FROM rust:1.62-slim-buster as build
# RUN apk add --no-cache musl-dev
WORKDIR /tmp/pet-monitor-app

# Separate layer for dependencies
COPY ./Cargo.* .
RUN mkdir ./src/ && touch ./src/lib.rs
RUN cargo build

COPY . .
# update modification time so it rebuilds
RUN touch -m ./src/lib.rs
RUN cargo build

FROM debian:buster-slim
COPY --from=build /tmp/pet-monitor-app/target/debug/pet-monitor-app /usr/local/bin/

COPY ./Rocket.toml /usr/local/etc/pet-monitor-app/
ENV ROCKET_CONFIG=/usr/local/etc/pet-monitor-app/Rocket.toml

ENTRYPOINT [ "pet-monitor-app" ]
EXPOSE 8000
