# Build image
FROM rust:slim-buster as build

# Install the necessary dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    protobuf-compiler

# Add proto files
WORKDIR /proto
COPY ../proto .

# Create empty project
WORKDIR /peernode

# Copy in files
COPY peernode/Cargo.toml .
COPY peernode/Cargo.lock .
COPY peernode/build.rs .
COPY peernode/src src

# Build the project with caching
RUN --mount=type=cache,target=/usr/local/cargo/registry \
	--mount=type=cache,target=/peernode/target \
	cargo build --release && \
    cp ./target/release/peer-node ./peer-node

# Runner image
FROM debian:buster-slim

# Install SSL
RUN apt-get update && apt-get install -y \
    libssl1.1

# Copy the binary from the build image
COPY --from=build /peernode/peer-node .

# Environment variables
ENV ORCA_MARKET_ADDRESS localhost:50051
ENV ORCA_EXTERNAL_IP 127.0.0.1
ENV ORCA_HTTP_PORT 8080

# Run the binary
CMD ./peer-node producer add files 100 && ./peer-node producer register $ORCA_HTTP_PORT -m $ORCA_MARKET_ADDRESS -i $ORCA_EXTERNAL_IP