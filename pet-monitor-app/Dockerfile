FROM rust:1.63-slim-bullseye as base
RUN apt-get install \
gstreamer1.0-plugins-base \
gstreamer1.0-plugins-good \
gstreamer1.0-plugins-bad \
gstreamer1.0-plugins-ugly \
libv4l-0 \
libv4l-dev

FROM base as build
ARG target=""

WORKDIR /usr/local/src/pet-monitor-app
COPY . ../client/dist ./
RUN [ $target = "" ] && cargo build --release || cargo build --release --target $target

FROM debian:bullseye-slim
RUN apt-get install \
gstreamer1.0-plugins-base \
gstreamer1.0-plugins-good \
gstreamer1.0-plugins-bad \
gstreamer1.0-plugins-ugly \
libv4l-0 \
libv4l-dev

COPY --from=build /usr/local/src/pet-monitor-app/target/release/pet-monitor-app /usr/local/bin

EXPOSE 80 443
ENTRYPOINT [ "pet-monitor-app" ]
