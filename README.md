# portfwd

A simple TCP and UDP port forwarder. It can be used to forward a port on the local machine to
another port on another host.

## Installation

### Via Nix 

```sh
# run without installing
nix run github:Wybxc/portfwd -- -f 172.18.80.1:80

# install to global profile
nix profile install github:Wybxc/portfwd
```

### Via Cargo

```sh
cargo install --git https://github.com/Wybxc/portfwd.git
```

## Usage

```text
Usage: portfwd [OPTIONS] --forward <FORWARD>

Options:
-p, --port <PORT>        The port to listen on, defaults to the same as the forward port
-f, --forward <FORWARD>  The address and port to forward to
-t, --tcp                Only enable TCP forwarding
-u, --udp                Only enable UDP forwarding
-T, --threads <THREADS>  Number of threads to use, defaults to the number of logical CPUs
-v...                    Verbose output (-v, -vv, etc.)
-h, --help               Print help
-V, --version            Print version
```

## Examples

Forward TCP & UDP port 80 to port 80 on the host 172.18.80.1:

```sh
portfwd -f 172.18.80.1:80
```

Forward TCP port 8080 to port 80 on the host 172.18.80.1:

```sh
portfwd -p 8080 -f 172.18.80.1:80 --tcp
```

Forward UDP port 53 to port 1053 on localhost:

```sh
portfwd -p 53 -f 127.0.0.1:1053 --udp
```