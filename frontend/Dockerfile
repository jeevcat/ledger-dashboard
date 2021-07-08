# Stage 1 - the build process
FROM node:15-alpine as build-deps
WORKDIR /src
COPY package.json yarn.lock ./
RUN yarn
COPY . ./
RUN yarn build

# Stage 2 - the production environment
FROM nginx:stable-alpine
COPY --from=build-deps /src/build /usr/share/nginx/html
RUN rm /etc/nginx/conf.d/default.conf
COPY nginx.conf /etc/nginx/conf.d

RUN apk add --no-cache nodejs npm
RUN npm install --global @beam-australia/react-env

COPY entrypoint.sh /var/entrypoint.sh
RUN chmod +x /var/entrypoint.sh
WORKDIR /usr/share/nginx/html
ENTRYPOINT ["/var/entrypoint.sh"]

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]