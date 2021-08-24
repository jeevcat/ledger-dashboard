# Stage 1: Build
FROM rust:1.53 as build
ENV PKG_CONFIG_ALLOW_CROSS=1

# Get hledger version 1.22
WORKDIR /usr/bin/hledger
RUN wget https://github.com/simonmichael/hledger/releases/download/1.22.2/hledger-linux-static-x64.zip
RUN unzip hledger-linux-static-x64.zip
RUN mv hledger-static-linux-x64 hledger
RUN mv hledger-web-static-linux-x64 hledger-web
RUN chmod +x ./hledger
RUN chmod +x ./hledger-web

# Set the working directory INSIDE the container
WORKDIR /usr/src/backend

# Copy files next to Dockerfile into WORKDIR
COPY . .

# Run cargo inside the container
RUN cargo install --path .

# Strip symbols from binary
RUN strip /usr/local/cargo/bin/backend

# Stage 2: Production
FROM debian:bullseye-slim

#RUN apt-get update && apt-get install -y locales libtinfo-dev libtinfo5 libssl1.1 ca-certificates && rm -rf /var/lib/apt/lists/*
RUN apt-get update && apt-get install -y libssl1.1 ca-certificates && rm -rf /var/lib/apt/lists/*

# Set the locale (needed for hledger)
#RUN sed -i '/en_US.UTF-8/s/^# //g' /etc/locale.gen && locale-gen
#ENV LANG en_US.UTF-8
#ENV LANGUAGE en_US:en
#ENV LC_ALL en_US.UTF-8

COPY --from=build /usr/bin/hledger/hledger                  /usr/local/bin
COPY --from=build /usr/bin/hledger/hledger-web              /usr/local/bin
COPY --from=build /usr/local/cargo/bin/backend              /usr/local/bin

ENTRYPOINT ["/usr/local/bin/backend"]