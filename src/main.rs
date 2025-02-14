// This is free and unencumbered software released into the public domain.

#![deny(unsafe_code)]

mod feature;

use clientele::{
    crates::clap::{Parser, Subcommand},
    exit, StandardOptions,
    SysexitsError::*,
};
use std::{
    env::consts::EXE_SUFFIX,
    os::unix::process::ExitStatusExt,
    path::{Path, PathBuf},
    process::Stdio,
};

/// ASIMOV Command-Line Interface (CLI)
#[derive(Debug, Parser)]
#[command(name = "ASIMOV", long_about)]
#[command(subcommand_required = true)]
#[command(arg_required_else_help = true)]
#[command(allow_external_subcommands = true)]
struct Options {
    #[clap(flatten)]
    flags: StandardOptions,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[clap(external_subcommand)]
    External(Vec<String>),
}

pub fn main() {
    // Load environment variables from `.env`:
    clientele::dotenv().ok();

    // Expand wildcards and @argfiles:
    let Ok(args) = clientele::args_os() else {
        exit(EX_USAGE);
    };

    // Parse command-line options:
    let options = Options::parse_from(&args);

    // Print the version, if requested:
    if options.flags.version {
        println!("ASIMOV {}", env!("CARGO_PKG_VERSION"));
        exit(EX_OK);
    }

    // Print the license, if requested:
    if options.flags.license {
        print!("{}", include_str!("../UNLICENSE"));
        exit(EX_OK);
    }

    // Configure debug output:
    if options.flags.debug {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    // Locate the given subcommand:
    let Command::External(command) = &options.command;
    let Some(command_path) = find_external_subcommand(&command[0]) else {
        eprintln!(
            "{}: command not found: asimov-{}{}",
            "asimov", command[0], EXE_SUFFIX
        );
        exit(EX_UNAVAILABLE);
    };

    // Execute the given subcommand:
    let status = std::process::Command::new(command_path)
        .args(&command[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Err(error) => {
            if options.flags.debug {
                eprintln!("{}: {}", "asimov", error);
            }
            exit(EX_SOFTWARE);
        }
        Ok(status) => {
            use std::process::exit;
            #[cfg(unix)]
            {
                if let Some(signal) = status.signal() {
                    if options.flags.debug {
                        eprintln!("{}: terminated by signal {}", "asimov", signal);
                    }
                    exit((signal | 0x80) & 0xff);
                }
            }
            exit(status.code().unwrap_or(EX_SOFTWARE.as_i32()))
        }
    }
}

fn find_external_subcommand(command: &str) -> Option<PathBuf> {
    let command_exe = format!("asimov-{}{}", command, EXE_SUFFIX);
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|path| path.join(&command_exe))
            .find(|path| is_executable(path))
    })
}

#[cfg(unix)]
fn is_executable(path: impl AsRef<Path>) -> bool {
    use std::os::unix::prelude::*;
    std::fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_executable(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_file()
}
