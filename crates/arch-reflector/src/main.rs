use arch_reflector::{Cli, run};
use clap::Parser;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    run(&cli).await;
}
