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
enum LoadSubcommand {
    /// Load overlay into the system
    Overlay {
        /// Overlay `FILE` to be loaded (typically .dtbo)
        file: String,

        /// `HANDLE` for the overlay directory which will be created
        /// under "/sys/kernel/config/device-tree/overlays"
        #[arg(long = "handle")]
        handle: Option<String>,
    },
    /// Load bitstream into the system
    Bitstream {
        /// Bitstream `FILE` to be loaded (typically .bit.bin)
        file: String,
    },
}

#[derive(Subcommand, Debug)]
enum RemoveSubcommand {
    /// Remove overlay with the `HANDLE` provided
    Overlay {
        /// `HANDLE` is the handle that is given during `load` operation
        /// it is different than device_handle which is being used for platform
        /// detection logic.
        #[arg(long = "handle")]
        handle: String,
    },
    /// Remove bitstream loaded in given `HANDLE` to fpga command
    Bitstream,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Get the status information for the given device handle
    Status,
    /// Load a bitstream or an overlay for the given device handle
    Load {
        #[command(subcommand)]
        command: LoadSubcommand,
    },
    /// Remove bitstream or an overlay
    Remove {
        #[command(subcommand)]
        command: RemoveSubcommand,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = Cli::parse();
    debug!("parsed cli command with {cli:?}");
    match cli.command {
        Commands::Status => {
            todo!()
        }
        Commands::Load { .. } => todo!(),
        Commands::Remove { .. } => todo!(),
    }
}
