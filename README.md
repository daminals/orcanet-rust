# Orcanet Rust

## Setup

1. Install [Rust](https://www.rust-lang.org/tools/install)
2. Install protoc:

   `apt install protobuf-compiler`

   (May require more a [more recent version](https://grpc.io/docs/protoc-installation/#install-pre-compiled-binaries-any-os))

## Running

To run the market server:

```Shell
cargo run
```

To run a test client:

```Shell
cargo run --bin test_client
```

(currently the Go test client is interoperable)

Note: currently requires two market servers running to have the Kademia network
operable.

```Shell
cargo run
# in another terminal, '-x' only launches kad network and not market server,
# to avoid launching a server on conflicting ports
cargo run -- -x
# in another terminal
cargo run --bin test_client
```

## API
Detailed gRPC endpoints are in `proto/market.proto`

- Holders of a file can register the file using the RegisterFile RPC.
  - Provide a User with 5 fields: 
    - `id`: some string to identify the user.
    - `name`: a human-readable string to identify the user
    - `ip`: a string of the public ip address
    - `port`: an int32 of the port
    - `price`: an int64 that details the price per mb of outgoing files
  - Provide a fileHash string that is the hash of the file
  - Returns nothing

- Then, clients can search for holders using the CheckHolders RPC
  - Provide a fileHash to identify the file to search for
  - Returns a list of Users that hold the file.



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

```bash
cd peernode
cargo run -- -p  [ --ip <IP - USE 127.0.0.1 FOR LOCAL TESTING> ]
```

To run the consumer:
```bash
cd peernode
cargo run -- -f <FILE_HASH>
```

## Running with Docker
We also provide a Docker compose file to easily run the producer and market server together. To run it:
```bash
docker-compose build
docker-compose up
```
This will automatically mount the local `peernode/files` directory to the producer container and expose the producer HTTP and market server gRPC ports.

