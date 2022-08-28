FROM node:18-alpine as base
RUN corepack enable

FROM base
WORKDIR /usr/local/src/pet-monitor-app
RUN pnpm install
RUN pnpm build
