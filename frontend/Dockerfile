# Stage 1 - the build process
FROM node:15-alpine as build-deps
WORKDIR /src
COPY package.json yarn.lock ./
RUN yarn
COPY . ./
RUN yarn build

# Stage 2 - the production environment
FROM nginx:1.19-alpine
COPY --from=build-deps /src/build /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]