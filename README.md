# Orcanet Rust

## Setup

1. Install [Rust](https://www.rust-lang.org/tools/install)
2. Install protoc:

   `apt install protobuf-compiler`

   (May require more a [more recent version](https://grpc.io/docs/protoc-installation/#install-pre-compiled-binaries-any-os))

## API

See [server/README.md](server/README.md) for the API documentation.

## Running

### Market Server
```Shell
cd market
cargo run
```

To run a test client:

```Shell
cd market
cargo run --bin test_client
```

(currently the Go test client is interoperable)

### Peer Node

To run the producer:
```bash
cd peernode
cargo run producer add <FILE_PATH> <PRICE>
cargo run producer register
```

To run the consumer:
```bash
cd peernode
cargo run consumer ls <FILE_HASH>
cargo run consumer get <FILE_HASH> <CHOSEN_PRODUCER>
```

Additional commands can be listed with the help command.

## CLI Interface

To set up a market connection, set the `boot_nodes` configuration:

```shell
market set -b /ip4/130.245.173.204/tcp/6881/p2p/QmSzkZ1jRNzM2CmSLZwZgmC9ePa4t2ji3C8WuffcJnb8R
```

In order to provide a market server node, set `public_address`

```shell
market set -p /ip4/0.0.0.0/tcp/6881
```

Demo:

```shell
producer add files/giraffe.jpg 1
producer register
producer ls
# new instance
consumer ls 908b7415fea62428bb69eb01d8a3ce64190814cc01f01cae0289939e72909227
# make sure you're on a public ip (or edit producer/register_files)
consumer get 908b7415fea62428bb69eb01d8a3ce64190814cc01f01cae0289939e72909227 {producer_id}
```

To test with HTTP server with local market: (file paths relative to where this is run)

```shell
cargo run --bin peer-node-server -F test_local_market
```

## Running with Docker
We also provide a Docker compose file to easily run the producer and market server together. To run it:
```bash
docker-compose build
docker-compose up
```
This will automatically mount the local `peernode/files` directory to the producer container and expose the producer HTTP and market server gRPC ports.

