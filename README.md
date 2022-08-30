# Async STD Rust server with rate limit

1. [What](#what)
2. [Protocol](#protocol)
3. [Server](#server)
4. [Client](#client)
5. [Reverse engineering](#reverseEngineering)
6. [Running Everything](#runningEverything)

## What

TCP server with rate limit within partition.
Partitions are defined on startup by splitting alphabeth `A-Z` to a configured number of groups.

## Protocol

Wire protocol between client & server is a simple line protocol:

```
request: [A-Z]\n
response: [0|1]\n
```

Request is requested symbol; `0` response means rate limit is not breached, `1` -- rate limit is breached.

Other payloads considered invalid, however server returns `1` if it receives malformed message.

## Server

Server is located in `server` and is implemented in `Rust` using `async-std` and `governor` libraries.

Server binary accepts just two positional arguments: NUMBER_OF_PARTITIONS and RATE_LIMIT.

Server understands two env variables:
* `SERVER_ADDRESS` -- bind address for a server (default: `127.0.0.1:31337`).
* `SERVER_SEED` -- seed for random generator.

To run server go to `server` and use `cargo run [ARGS...]` or build docker image using provided `Dockerfile`.

## Client

Client is located in `clients/client.py` and is implemented in `Python` (`> 3.6`). Only std-lib is used.
To run use `python clients.py` or execute it with `./clients.py`.
`argparse` is used to implemented subcommands, so help will be provided.

Number of different behaviours are implemented, some of it are:
* `cycle` -- cycles through available symbols with configured delay.
* `scenario` -- parses string argument as a sequence of messages and delays and executes that scenario.
* `find_partitions` -- runs reverse engineering algorithm, trying to figure out partitioning used by server.

Full list of supported behaviours is shown in program usage.

## Reverse Engineering

`reverse-engineering/prototype.py` contains network-less implementation of reverse engineering algo used in `find_partitions` client behaviour above.

## Running everything

`docker-compose up` builds `server` and `client` and runs `find_partitions` behaviours.
