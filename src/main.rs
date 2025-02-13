// This is free and unencumbered software released into the public domain.

#![deny(unsafe_code)]

mod exit;
mod feature;

use crate::exit::ExitCode;
use clientele::{crates::clap::Parser, StandardOptions};

/// ASIMOV Command-Line Interface (CLI)
#[derive(Debug, Parser)]
#[command(name = "ASIMOV", about)]
#[command(arg_required_else_help = true)]
struct Options {
    #[clap(flatten)]
    flags: StandardOptions,

    command: String,
}

pub fn main() -> Result<(), ExitCode> {
    // Load environment variables from `.env`:
    clientele::dotenv().ok();

    // Expand wildcards and @argfiles:
    let args = clientele::args_os()?;

    // Parse command-line options:
    let options = Options::parse_from(args);

    if options.flags.version {
        println!("ASIMOV {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if options.flags.license {
        print!("{}", include_str!("../UNLICENSE"));
        return Ok(());
    }

    // Configure verbose/debug output:
    if options.flags.verbose > 0 || options.flags.debug {
        // TODO: configure tracing
    }

    Ok(()) // TODO
}
