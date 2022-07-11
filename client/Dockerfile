# syntax=docker/dockerfile:1

# build frontend
#FROM node:18-alpine AS build
#WORKDIR /tmp/pet-monitor-app
#COPY . .
#RUN yarn install --immutable --immutable-cache
#RUN yarn build

# set up nginx
FROM nginx:1.21-alpine
COPY nginx.conf /etc/nginx
#COPY --from=build /tmp/pet-monitor-app/build /etc/nginx/html
COPY ./build /etc/nginx/html
COPY ./pet-monitor-app.pem /etc/ssl/certs
COPY ./pet-monitor-app.key /etc/ssl/certs

ENTRYPOINT [ "nginx" ]
EXPOSE 80 443
