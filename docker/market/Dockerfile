# Build image
FROM rust:slim-buster as build

# Install the necessary dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    protobuf-compiler \
    git

# Add proto files
WORKDIR /proto
COPY ../proto .

# Create empty project
WORKDIR /market

# Copy in files
COPY market/Cargo.toml .
COPY market/Cargo.lock .
COPY market/lib-proto lib-proto
COPY market/src src

# Build the project with caching
RUN --mount=type=cache,target=/usr/local/cargo/registry \
	--mount=type=cache,target=/market/target \
	cargo build --release && \
    cp ./target/release/orcanet-market-ferrous ./orcanet-market-ferrous

# Runner image
FROM debian:buster-slim

# Install SSL
RUN apt-get update && apt-get install -y \
    libssl1.1

# Copy the binary from the build image
COPY --from=build /market/orcanet-market-ferrous .

# Run the binary
CMD ./orcanet-market-ferrous