FROM rust:1.63-slim-bullseye as base
RUN apt-get update
# install glib
RUN apt-get install -y libgtk-3.0
# install gstreamer
RUN apt-get install -y libgstreamer1.0-0 libgstreamer1.0-dev
RUN apt-get install -y \
gstreamer1.0-plugins-base \
gstreamer1.0-plugins-good \
gstreamer1.0-plugins-bad \
gstreamer1.0-plugins-ugly
# install libv4l2
RUN apt-get install -y libv4l-0 libv4l-dev
# install pkg-config (for gstreamer)
RUN apt-get install -y pkg-config

FROM base as build
# optional target triple argument for cross-compilation
ARG target=""
WORKDIR /usr/local/src/pet-monitor-app

# only recompile dependencies if Cargo.toml or Cargo.lock change
COPY ./Cargo.toml ./Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN [ "$target" = "" ] && cargo build --release || cargo build --release --target $target
COPY ./src ./dist ./
# update modification time so cargo recompiles main.rs
RUN touch -m src/main.rs
RUN [ "$target" = "" ] && cargo build --release || cargo build --release --target $target

# copy binary to target/release so the COPY in the last stage works when cross-compiling
RUN [ "$target" = "" ] || cp ./target/$target/release/pet-monitor-app ./target/release

FROM base
COPY --from=build /usr/local/src/pet-monitor-app/target/release/pet-monitor-app /usr/local/bin
COPY ./pet-monitor-app.toml /usr/local/etc/pet-monitor-app

EXPOSE 80 443
ENTRYPOINT [ "pet-monitor-app" ]
