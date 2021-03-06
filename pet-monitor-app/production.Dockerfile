# syntax=docker/dockerfile:1.4

# Alpine uses musl instead of glibc, which is slower
FROM rust:1.62-slim-bullseye as build
WORKDIR /tmp/pet-monitor-app

# Separate layer for dependencies
COPY ./Cargo.toml ./Cargo.lock ./
RUN mkdir ./src/ && touch ./src/lib.rs
RUN cargo build --release

COPY . .
# update modification time so it rebuilds
RUN touch -m ./src/lib.rs
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=build /tmp/pet-monitor-app/target/release/pet-monitor-app /usr/local/bin/

COPY ./Rocket.toml /usr/local/etc/pet-monitor-app/
ENV ROCKET_CONFIG=/usr/local/etc/pet-monitor-app/Rocket.toml

ENTRYPOINT [ "pet-monitor-app" ]
EXPOSE 8001
