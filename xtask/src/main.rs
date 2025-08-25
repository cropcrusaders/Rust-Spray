use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process::Command;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run fmt, clippy and tests for local development
    Dev,
    /// Build release binaries with timing information
    BenchBuild,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Dev => {
            run("cargo", &["fmt"])?;
            run(
                "cargo",
                &[
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--",
                    "-D",
                    "warnings",
                ],
            )?;
            run("cargo", &["test"])?;
        }
        Commands::BenchBuild => {
            run("cargo", &["build", "--release", "-Z", "timings"])?;
            run("cargo", &["size", "-A", "target/release/rustspray"])?;
        }
    }
    Ok(())
}

fn run(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd).args(args).status()?;
    if !status.success() {
        anyhow::bail!("command `{cmd}` failed");
    }
    Ok(())
}
