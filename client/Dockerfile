FROM node:18-alpine as base
EXPOSE 5173

FROM node:18-alpine
WORKDIR /usr/local/src/pet-monitor-app
RUN pnpm install
RUN pnpm build
