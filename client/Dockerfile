# syntax=docker/dockerfile:1

FROM node:18-alpine
WORKDIR /usr/local/src/pet-monitor-app
COPY . .
RUN yarn install

ENTRYPOINT [ "yarn", "start" ]
EXPOSE 3000
