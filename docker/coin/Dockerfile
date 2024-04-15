# Build image
FROM rust:slim-bookworm as build

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
WORKDIR /coin-mock

# Copy in files
COPY coin-mock/Cargo.toml .
COPY coin-mock/Cargo.lock .
COPY coin-mock/build.rs .
COPY coin-mock/src src

# Build the project with caching
RUN --mount=type=cache,target=/usr/local/cargo/registry \
	--mount=type=cache,target=/coin-mock/target \
	cargo build --release && \
    cp ./target/release/orcanet-coin ./orcanet-coin

# Runner image
FROM debian:bookworm-slim

# Install SSL
#RUN apt-get update && apt-get install -y \
#    libssl1.1

# Copy the binary from the build image
COPY --from=build /coin-mock/orcanet-coin .

# Run the binary
CMD ./orcanet-coin