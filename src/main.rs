mod program;
mod tape;
mod errors;
mod parser;

use clap::{Parser, Subcommand, Args};
use program::Program;
use tape::Tape;

/// BrainF*ck Easy Mode (BFEM). Brainf*ck with quality-of-life improvements.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[command(flatten)]
    disable_flags: DisableFlags,

    #[command(flatten)]
    tape_flags: TapeFlags,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile the given source file to the given output file.
    Compile(CompileArgs),
    /// Run the given file
    Run(RunArgs),
    /// Show a detailed preview of parser info
    Explain(RunArgs)
}

#[derive(Args)]
struct CompileArgs {
    path: std::path::PathBuf,
    
    output: std::path::PathBuf,

    /// Output instruction tree (and then exit)
    #[arg(short, long)]
    tree: bool,
}

#[derive(Args)]
struct RunArgs {
    path: std::path::PathBuf,
}

#[derive(Args, Clone, Copy)]
pub struct DisableFlags {
    /// Disable variable aliases
    #[arg(long)]
    disable_aliases: bool,
    /// Disable consecutive instruction optimisations
    #[arg(long)]
    disable_optimise: bool,
    /// Disable alias pre-allocation
    #[arg(long)]
    disable_alloc: bool,
}

#[derive(Args)]
pub struct TapeFlags {
    #[arg(long, value_enum, default_value_t=tape::TapeMode::Circular)]
    tape_mode: tape::TapeMode,
    #[arg(long, value_enum, default_value_t=tape::CellMode::Circular)]
    cell_mode: tape::CellMode,
    #[arg(long, default_value_t = 30000)]
    tape_size: u128,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Compile(args) => {
            let mut program = Program::read_file(args.path.clone(), Tape::new(cli.tape_flags), cli.disable_flags);
            
            println!("{:?}", program.get_instructions());
        },
        Commands::Run(args) => {
            let mut program = Program::read_file(args.path.clone(), Tape::new(cli.tape_flags), cli.disable_flags);
            program.setup();

            program.run();
        }
        Commands::Explain(args) => {
            let mut program = Program::read_file(args.path.clone(), Tape::new(cli.tape_flags), cli.disable_flags);

            program.info();
        }
    }
}