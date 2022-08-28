FROM node:18-alpine as base
RUN corepack enable

FROM base
WORKDIR /usr/local/src/pet-monitor-app
COPY . .
RUN pnpm install
RUN pnpm build
