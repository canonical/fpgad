use clap::{Parser, Subcommand, arg, command};
use log::debug;

#[derive(Parser, Debug)]
#[command(name = "fpga")]
#[command(bin_name = "fpga")]
struct Cli {
    #[arg(
        long = "handle",
        help = r#"fpga device handle to be used for operations.
Default value for this option is calculated in runtime and application
picks first available fpga in the system (under /sys/class/fpga_manager).
        "#
    )]
    handle: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Status,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = Cli::parse();
    debug!("parsed cli command with {cli:?}");
    match cli.command {
        Commands::Status => {
            todo!()
        }
    }
}
