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
    if let Err(e) = match cli.command {
        Command::Code => codegen(&cli.path),
        Command::Docs => docgen(&cli.path),
    } {
        eprintln!("{e}");
    }
    Ok(())
}

fn codegen(path: &str) -> anyhow::Result<()> {
    let code = isf::codegen::generate_code(path)?;
    println!("{code}");
    Ok(())
}

fn docgen(_path: &str) -> anyhow::Result<()> {
    todo!();
}
