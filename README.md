# Orcanet Rust

## Requirements

The `setup.sh` script provided should install dependencies and build the project
(tested on Ubuntu 20.04). Otherwise,

1. Install [Rust](https://www.rust-lang.org/tools/install)
2. Install protoc:

   `apt install protobuf-compiler`

   (May require more a [more recent version](https://grpc.io/docs/protoc-installation/#install-pre-compiled-binaries-any-os))

## Dht API

- The market stores file metadata in a FileInfo struct, with key
  `FileInfo/{file_hash}`
  - `file_hash`: the SHA-256 hash of the file contents
  - `chunk`: `Vec<(String, u64)>` a list of file hashes and sizes for each chunk
  of the file
  - `suppliers`: `Vec<(User, u64)>`: a list of users supplying the file and their
  expiration times
- `User`
  - `id`: some string to identify the user.
  - `name`: a human-readable string to identify the user
  - `ip`: a string of the public ip address
  - `port`: an int32 of the port
  - `price`: an int64 that details the price per mb of outgoing files

- Then, clients can search for holders using the CheckHolders RPC
  - Provide a fileHash to identify the file to search for
  - Returns a list of Users that hold the file.

## Running with Docker (Deprecated)

We provide a Docker compose file to easily run the producer and market server
together. To run it:

```bash
docker compose build
docker compose up
```

This will automatically mount the local `peernode/files` directory to the
producer container and expose the producer HTTP and market server gRPC ports.

The market server requires another Kademlia node to be connected to it in order
to store data. In order to bootstrap to another node, pass an address to docker
compose, either by the `.env` file, or on the command line.

```bash
BOOTSTRAP="/ip4/{addr}/tcp/6881/p2p/{peer_id} docker compose up --build
```

Otherwise, the market server will launch a new network, which requires at least
one other node to connect before it is able to store data.

## Running Without Docker

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

## CLI Interface

To set up a market connection, set the `bootstrap_peers` configuration:

```shell
market set -b /ip4/130.245.173.204/tcp/6881/p2p/QmSzkZ1jRNzM2CmSLZwZgmC9ePa4t2ji3C8WuffcJnb8R
```

In order to provide a market server node, set `listen_address` and `private-key`

```shell
market set -l /ip4/0.0.0.0/tcp/6881 -k private.pk8
```

Demo:

```shell
producer add files/giraffe.jpg 1
producer register
# new instance
consumer ls 8c679fbaa3a384196adf87937b58c893304370c5b35f77f83cdb897b030e8fc2
# make sure you're on a public ip (or edit producer/register_files)
consumer get 8c679fbaa3a384196adf87937b58c893304370c5b35f77f83cdb897b030e8fc2 {producer_id}

### Market Connection

The peer node requires its market to be configured in order to query the network
or share data.

* `bootstrap-peers`
  * Space separated list of Multiaddr peer nodes to connect to in order to
  bootstrap the node onto a Kademlia network.
  * *If this is not provided, the application will start a new Kademlia network*
  * **The provided peer nodes cannot be on the same local network in order for**
  **the node to act as a server**
* `private-key`
  * Private key in order for the node to be set up as a Kademlia server node.
  * The application will print out the peer id derived from this key.
  * This must be provided in order for the node to **act as a server node**,
  otherwise it will only act as a client node (it can only query the network,
  and not provide data).
* `listen-address`
  * Multiaddr that the application will listen on to act as a Kademlia server node.

```shell
market set -b /ip4/{ip_address}/tcp/6881/{peer_id_hash} -k private.pk8 -l /ip4/0.0.0.0/tcp/6881
```

## Market Server

The `dht_client` binary will only run the Kademlia node.

### Parameters

The `dht_client` binary requires these parameters to configure the
Kademlia node running on the application.

* `bootstrap-peers`
* `private-key`
* `listen-address`
  * By default, *if `private-key` is provided*, the node will listen on
  `/ip4/0.0.0.0/tcp/6881`

#### Start a new Kademlia network

To start a new Kademlia network, first create a public/private key pair

```Shell
openssl genrsa -out private.pem 2048
openssl pkcs8 -in private.pem -inform PEM -topk8 -out private.pk8 -outform DER -nocrypt

rm private.pem      # optional
```

Then run the peer node or `dht_client` with both the `private-key` and
`listen-address` parameters provided.

```Shell
cargo run -- --private-key private.pk8 --listen-address /ip4/0.0.0.0/tcp/6881
cargo run --bin dht_client -- --private-key private.pk8 # by default, /ip4/0.0.0.0/tcp/6881
```

#### Run a market test client (deprecated)

```Shell
cargo run --bin test_client
```

