# Stage 1: Build
FROM rust:1.49 as build
ENV PKG_CONFIG_ALLOW_CROSS=1

# Get hledger version 1.20.4
WORKDIR /usr/bin/hledger
RUN wget https://github.com/simonmichael/hledger/releases/download/1.20.4/hledger-ubuntu.zip
RUN unzip hledger-ubuntu.zip
RUN chmod +x ./hledger-web

# Set the working directory INSIDE the container
WORKDIR /usr/src/backend

# Copy files next to Dockerfile into WORKDIR
COPY . .

# Run cargo inside the container
RUN cargo install --path .

# Stage 2: Production
FROM debian:buster-slim
RUN apt-get update && apt-get install -y libtinfo-dev libtinfo5 libssl1.1 && rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/bin/hledger /usr/local/bin
COPY --from=build /usr/local/cargo/bin/backend /usr/local/bin/backend

CMD ["backend"]