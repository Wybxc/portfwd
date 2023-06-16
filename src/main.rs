//! A simple TCP and UDP port forwarder. It can be used to forward a port on the local machine to
//! another port on another host.
//!
//! ## Usage
//!
//! ```text
//! Usage: portfwd [OPTIONS] --forward <FORWARD>
//!
//! Options:
//! -p, --port <PORT>        The port to listen on, defaults to the same as the forward port
//! -f, --forward <FORWARD>  The address and port to forward to
//! -t, --tcp                Only enable TCP forwarding
//! -u, --udp                Only enable UDP forwarding
//! -T, --threads <THREADS>  Number of threads to use, defaults to the number of logical CPUs
//! -v...                    Verbose output (-v, -vv, etc.)
//! -h, --help               Print help
//! -V, --version            Print version
//! ```
//!
//! ## Examples
//!
//! Forward TCP & UDP port 80 to port 80 on the host 172.18.80.1:
//!
//! ```sh
//! portfwd -f 172.18.80.1:80
//! ```
//!
//! Forward TCP port 8080 to port 80 on the host 172.18.80.1:
//!
//! ```sh
//! portfwd -p 8080 -f 172.18.80.1:80 --tcp
//! ```
//!
//! Forward UDP port 53 to port 1053 on localhost:
//!
//! ```sh
//! portfwd -p 53 -f 127.0.0.1:1053 --udp
//! ```

use std::net::{SocketAddr, TcpListener, TcpStream};

use clap::Parser;
use easy_parallel::Parallel;
use smol::{channel::unbounded, future, io, Async, Executor};

mod cli;

/// Starts a TCP server that forwards messages from clients to the destination.
#[tracing::instrument]
async fn tcp_server(port: u16, forward: SocketAddr) -> io::Result<()> {
    // Create a listener.
    let listener = Async::<TcpListener>::bind(([127, 0, 0, 1], port))?;
    tracing::info!("Listening on {}", listener.get_ref().local_addr()?);

    // Accept clients in a loop.
    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let (reader, writer) = io::split(stream);
        tracing::info!("Accepted client: {}", peer_addr);

        // Connect to the destination.
        let dest = Async::<TcpStream>::connect(forward).await?;
        let dest_peer_addr = dest.get_ref().peer_addr()?;
        let (dest_reader, dest_writer) = io::split(dest);
        tracing::debug!("Connected to destination: {}", dest_peer_addr);

        // Spawn a task that copies messages from the client to the destination.
        smol::spawn(async move {
            io::copy(reader, dest_writer).await?;
            tracing::info!("Client closed connection: {}", peer_addr);
            Ok(()) as io::Result<()>
        })
        .detach();

        // Spawn a task that copies messages from the destination to the client.
        smol::spawn(async move {
            io::copy(dest_reader, writer).await?;
            tracing::debug!("Destination closed connection: {}", dest_peer_addr);
            Ok(()) as io::Result<()>
        })
        .detach();
    }
}

/// Starts a UDP server that forwards messages from clients to the destination.
#[tracing::instrument]
async fn udp_server(port: u16, forward: SocketAddr) -> io::Result<()> {
    // Create a listener.
    let socket = Async::<std::net::UdpSocket>::bind(([127, 0, 0, 1], port))?;
    tracing::info!("Listening on {}", socket.get_ref().local_addr()?);

    // Receive messages in a loop.
    loop {
        // Receive a message from the client.
        let mut buf = vec![0; 1024];
        let (size, peer_addr) = socket.recv_from(&mut buf).await?;
        tracing::info!("Received {} bytes from {}", size, peer_addr);

        // Send the message to the destination.
        socket.send_to(&buf[..size], forward).await?;
        tracing::info!("Sent {} bytes to {}", size, forward);
    }
}

#[tracing::instrument]
fn main() -> io::Result<()> {
    // Parse command line arguments.
    let cli = cli::Cli::parse();

    // Initialize tracing.
    let verbose = match cli.verbose {
        0 => tracing::Level::INFO,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };
    tracing_subscriber::fmt().with_max_level(verbose).init();

    // The port to listen on, defaults to the same as the forward port.
    let port = cli
        .port
        .map(u16::from)
        .unwrap_or_else(|| cli.forward.port());
    tracing::debug!(port);

    // The address and port to forward to.
    let forward = cli.forward;
    tracing::debug!(?forward);

    // Enable TCP and/or UDP forwarding.
    let (tcp, udp) = if !cli.features.tcp && !cli.features.udp {
        (true, true)
    } else {
        (cli.features.tcp, cli.features.udp)
    };
    tracing::debug!(tcp, udp);

    // Number of threads to use, defaults to the number of logical CPUs.
    let threads = cli.threads.unwrap_or_else(num_cpus::get);
    tracing::debug!(threads);

    // Start a TCP server.
    let tcp_server = if tcp {
        smol::spawn(tcp_server(port, forward))
    } else {
        smol::spawn(async { Ok(()) })
    };

    // Start a UDP server.
    let udp_server = if udp {
        smol::spawn(udp_server(port, forward))
    } else {
        smol::spawn(async { Ok(()) })
    };

    // Wait for the servers to finish.
    let ex = Executor::new();
    let (signal, shutdown) = unbounded::<()>();

    Parallel::new()
        // Run executor threads.
        .each(0..threads, |i| {
            let _ = future::block_on(ex.run(shutdown.recv()));
            tracing::debug!("Executor thread {} finished", i);
        })
        // Run the main future on the current thread.
        .finish(|| {
            future::block_on(async {
                tcp_server.await?;
                udp_server.await?;
                drop(signal);
                Ok(()) as io::Result<()>
            })
        })
        .1
}
