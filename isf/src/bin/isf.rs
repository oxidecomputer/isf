// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to an ISF spec
    path: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate code from an ISF spec
    Code,
    /// Generate docs from an ISF spec
    Docs,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Code => codegen(&cli.path),
        Command::Docs => docgen(&cli.path),
    }
}

fn codegen(path: &str) -> anyhow::Result<()> {
    let code = isf::codegen::generate_code(path)?;
    println!("{code}");
    Ok(())
}

fn docgen(_path: &str) -> anyhow::Result<()> {
    todo!();
}
