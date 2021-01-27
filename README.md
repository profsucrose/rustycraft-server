# rustycraft-server

Server for RustyCraft using Rust + TCP sockets

## Usage

To use, simply clone the repo and run `cargo run --release`:
```
git clone https://github.com/profsucrose/rustycraft-server.git && cd rustycraft-server && cargo run --release
```

A RustyCraft server will be hosted on the port 25566 by default. This is the default port a client will join if they do not specify a port number when connecting to a server address.

To specify a port, simply pass a valid port number as a flag: 
```
cargo run --release <port_number>
```

## Joining

To join a server on the client, click the "Connect to Server" to access the connect GUI and type in the address. Assuming the server is hosted successfully you should be able to click "Connect" and join. 