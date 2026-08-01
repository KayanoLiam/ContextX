mod config;
mod grok;
mod mcp;
mod server;

use config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load()?;
    server::run(config).await
}
