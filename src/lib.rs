pub mod assert;
pub mod cli;
pub mod config;
pub mod http;
pub mod report;
pub mod runner;
pub mod stats;

use clap::Parser;
use cli::Cli;

use crate::{
    cli::Commands,
    config::{TestConfig, ValidatedTestConfig},
};

pub async fn run() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Test(args) => {
            let config = load_config(&args.config);
            let results = runner::test::run(&config, args.parallel)
                .await
                .unwrap_or_else(|e| {
                    eprintln!("error: {}", e);
                    std::process::exit(1)
                });
            report::print_test_results(&results, &args.output);
        }
        Commands::Load(args) => {
            let config = load_config(&args.config);
            let result = runner::load::run(&config).await.unwrap_or_else(|e| {
                eprintln!("error: {}", e);
                std::process::exit(1)
            });
            report::print_load_results(&result, &args.output);
        }
        Commands::Run(args) => {
            let config = load_config(&args.config);
            let results = runner::test::run(&config, args.parallel)
                .await
                .unwrap_or_else(|e| {
                    eprintln!("error: {}", e);
                    std::process::exit(1)
                });
            report::print_test_results(&results, &args.output);
            let result = runner::load::run(&config).await.unwrap_or_else(|e| {
                eprintln!("error: {}", e);
                std::process::exit(1)
            });
            report::print_load_results(&result, &args.output);
        }
    }
}

fn load_config(path: &str) -> ValidatedTestConfig {
    TestConfig::from_file(path)
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            std::process::exit(1)
        })
        .validate()
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            std::process::exit(1)
        })
}
