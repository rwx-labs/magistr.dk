use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Opts {
    /// PostgreSQL URL
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,
}
