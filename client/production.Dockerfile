# syntax=docker/dockerfile:1.4

#FROM node:18-alpine AS build
#WORKDIR /tmp/pet-monitor-app
#COPY . .
#RUN yarn install --immutable --immutable-cache
#RUN yarn build

FROM nginx:1.23-alpine

ARG KEY_PATH="./pet-monitor-app.key"
ARG CERT_PATH="./pet-monitor-app.pem"

WORKDIR /etc/nginx

COPY ./nginx.conf .

#COPY --from=build /tmp/pet-monitor-app/build ./html
COPY ./build ./html/

COPY $KEY_PATH /etc/ssl/certs/pet-monitor-app.key
COPY $CERT_PATH /etc/ssl/certs/pet-monitor-app.pem

ENTRYPOINT [ "nginx", "-g", "daemon off;" ]
EXPOSE 80 443
