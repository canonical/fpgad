use clap::{Parser, Subcommand, arg, command};
use log::debug;

#[derive(Parser, Debug)]
#[command(name = "fpga")]
#[command(bin_name = "fpga")]
struct Cli {
    /// fpga device `HANDLE` to be used for the operations.
    /// Default value for this option is calculated in runtime and the application
    /// picks the first available fpga in the system (under /sys/class/fpga_manager)
    #[arg(long = "handle")]
    handle: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Get the status information for the given device handle
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
