use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

use cargo_typoguard::{check_dependencies, has_danger, parse_dependencies, print_results};

#[derive(Parser)]
#[command(
    name = "cargo-typoguard",
    about = "Check Cargo.toml dependencies for potential typosquatting",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check dependencies for potential typosquatting
    Check {
        /// Path to Cargo.toml file
        #[arg(long, default_value = "./Cargo.toml")]
        path: PathBuf,

        /// Similarity threshold (0.0-1.0). Dependencies with similarity above
        /// this threshold to a popular crate will be flagged.
        #[arg(long, default_value = "0.8")]
        threshold: f64,
    },
}

fn main() {
    // When invoked as `cargo typoguard`, cargo passes "typoguard" as the first arg.
    // Filter it out so clap can parse correctly.
    let args: Vec<String> = std::env::args().collect();
    let filtered_args: Vec<String> = if args.len() > 1 && args[1] == "typoguard" {
        std::iter::once(args[0].clone())
            .chain(args[2..].iter().cloned())
            .collect()
    } else {
        args
    };

    let cli = Cli::parse_from(filtered_args);

    match cli.command {
        Commands::Check { path, threshold } => {
            if threshold < 0.0 || threshold > 1.0 {
                eprintln!("Error: threshold must be between 0.0 and 1.0");
                process::exit(1);
            }

            let cargo_toml = match std::fs::read_to_string(&path) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error reading {}: {}", path.display(), e);
                    process::exit(1);
                }
            };

            let deps = match parse_dependencies(&cargo_toml) {
                Ok(deps) => deps,
                Err(e) => {
                    eprintln!("Error parsing Cargo.toml: {e}");
                    process::exit(1);
                }
            };

            if deps.is_empty() {
                println!("No dependencies found in {}", path.display());
                process::exit(0);
            }

            println!(
                "Checking {} dependencies from {} (threshold: {:.0}%)...",
                deps.len(),
                path.display(),
                threshold * 100.0
            );

            let results = check_dependencies(&deps, threshold, false);
            print_results(&results);

            if has_danger(&results) {
                process::exit(1);
            }
        }
    }
}
