use std::{net::SocketAddr, num::NonZeroU16};

use clap::{Args, Parser};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The port to listen on, defaults to the same as the forward port.
    #[clap(short, long)]
    pub port: Option<NonZeroU16>,

    /// The address and port to forward to.
    #[clap(short, long)]
    pub forward: SocketAddr,

    #[command(flatten)]
    pub features: Features,

    /// Number of threads to use, defaults to the number of logical CPUs.
    #[clap(short = 'T', long)]
    pub threads: Option<usize>,

    /// Verbose output (-v, -vv, etc.)
    #[clap(short, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Args)]
#[group(required = false, multiple = false)]
pub struct Features {
    /// Only enable TCP forwarding.
    #[clap(short, long)]
    pub tcp: bool,

    /// Only enable UDP forwarding.
    #[clap(short, long)]
    pub udp: bool,
}
