# syntax=docker/dockerfile:1

# Pet Montitor App
# Copyright (C) 2022  Samuel Nystrom
# 
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
# 
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
# 
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
