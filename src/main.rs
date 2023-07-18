use anyhow::Context;
use clap::Parser;

use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use huxi_server::config::{WebArgs, WebConfig};
use huxi_server::http;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "example_form=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = WebArgs::parse();

    let toml = std::fs::read_to_string(args.www_config)?;
    let config: WebConfig = toml::from_str::<WebConfig>(&toml)?;

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&config.database.url)
        .await
        .context("could not connect to database_url")?;

    http::serve(config, db, args.www_port).await?;

    Ok(())
}
