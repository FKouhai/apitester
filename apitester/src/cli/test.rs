use clap::Args;

#[derive(Args)]
pub struct TestArgs {
    /// Path to the test configuration YAML file
    pub config: String,

    /// Run requests in parallel
    #[arg(short, long)]
    pub parallel: bool,

    /// Output format: terminal, json, tap
    #[arg(short, long, default_value = "terminal")]
    pub output: String,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}
