# Orcanet Rust

## Setup

1. Install [Rust](https://www.rust-lang.org/tools/install)
2. Install protoc:

   `apt install protobuf-compiler`

   (May require more a [more recent version](https://grpc.io/docs/protoc-installation/#install-pre-compiled-binaries-any-os))

## Running

The market server requires a bootstrap Kademlia node to connect to. Skip this
step if you want to connect to an existing network.

To create a Kademlia network node, first create a public/private key pair

```Shell
openssl genrsa -out private.pem 2048
openssl pkcs8 -in private.pem -inform PEM -topk8 -out private.pk8 -outform DER -nocrypt

rm private.pem      # optional
```

Then start the swarm node

```Shell
cargo run --bin dht_swarm_start -- --private-key private.pk8 --listen-address /ip4/0.0.0.0/tcp/6881
```

Now we can start a market server

```Shell
cargo run -- --bootstrap-peers /ip4/{ip_addr}/tcp/{port}/p2p/{public key}
```

To run a test client

```Shell
cargo run --bin test_client
```

(currently the Go test client is interoperable)

To run more Kademlia nodes for testing

```Shell
cargo run --bin dht_client -- --bootstrap-peers /ip4/{ip_addr}/tcp/{port}/p2p/{public key}
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

Additional commands can be detailed by utilizing the help command.

## Running with Docker
We also provide a Docker compose file to easily run the producer and market server together. To run it:
```bash
docker-compose build
docker-compose up
```
This will automatically mount the local `peernode/files` directory to the producer container and expose the producer HTTP and market server gRPC ports.

