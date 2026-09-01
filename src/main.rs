mod application;
mod cli;
mod database;
mod domain;
mod repository;

use anyhow::Result;
use clap::Parser;

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let command = cli::Cli::parse();
    cli::run(command)
}
