pub mod load;
pub mod run;
pub mod test;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "apitester", about = "Declarative HTTP test & load runner")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Test(test::TestArgs),
    Load(load::LoadArgs),
    Run(run::RunArgs),
}
