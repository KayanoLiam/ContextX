use std::{error::Error, sync::Arc};

use axum::{Router, routing::get};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};

use crate::{config::AppConfig, grok::GrokClient, mcp::ContextXServer};

/// Streamable HTTP形式のMCPサーバーを起動します。
pub async fn run(config: AppConfig) -> Result<(), Box<dyn Error>> {
    let AppConfig {
        api_key,
        upstream_url,
        bind_addr,
        allowed_hosts,
    } = config;

    let grok_client = GrokClient::new(api_key, upstream_url)?;
    let service: StreamableHttpService<ContextXServer, LocalSessionManager> =
        StreamableHttpService::new(
            move || Ok(ContextXServer::new(grok_client.clone())),
            Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig::default().with_allowed_hosts(allowed_hosts),
        );

    let app = Router::new()
        .route("/health", get(health))
        .nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    println!("contextXを http://{bind_addr}/mcp で起動しました");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}
