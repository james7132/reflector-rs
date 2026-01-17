use arch_reflector::{Cli, run};
use clap::Parser;

fn main() {
    let cli = Cli::parse();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(cli.run.threads.max(1))
        .build()
        .unwrap()
        .block_on(run(&cli));
}
