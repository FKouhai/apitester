use clap::Args;

#[derive(Args)]
pub struct LoadArgs {
    /// Path to the test configuration YAML file
    pub config: String,

    /// Number of concurrent requests (overrides config)
    #[arg(short, long)]
    pub concurrency: Option<usize>,

    /// Duration of the load test (overrides config)
    #[arg(short, long)]
    pub duration: Option<String>,

    #[arg(short, long, default_value = "terminal")]
    pub output: String,
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}
