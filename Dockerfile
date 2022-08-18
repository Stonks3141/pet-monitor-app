# syntax=docker/dockerfile:1.4

# build static files
FROM node:18.6-alpine as client
WORKDIR /tmp/pet-monitor-app/client

COPY ./client .
RUN pnpm build

# Alpine uses musl instead of glibc, which is slower
FROM rust:1.62-slim-bullseye as build

# install libv4l2
RUN apt-get install libv4l-0 libv4l-dev

WORKDIR /tmp/pet-monitor-app/pet-monitor-app
COPY --from=client /tmp/pet-monitor-app/client/dist ../client

# Separate layer for dependencies
COPY ./pet-monitor-app/Cargo.toml ./pet-monitor-app/Cargo.lock ./
RUN mkdir ./src/ && touch ./src/lib.rs
RUN cargo build --release

COPY ./pet-monitor-app .
# update modification time so it rebuilds
RUN touch -m ./src/lib.rs
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=build /tmp/pet-monitor-app/target/debug/pet-monitor-app /usr/local/bin/

COPY ./Rocket.toml /usr/local/etc/pet-monitor-app/
ENV ROCKET_CONFIG=/usr/local/etc/pet-monitor-app/Rocket.toml

ENTRYPOINT [ "pet-monitor-app" ]
EXPOSE 8001
