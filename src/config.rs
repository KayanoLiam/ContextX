use std::{env, error::Error, fmt, sync::Arc};

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:3000";
const DEFAULT_ALLOWED_HOSTS: &str = "localhost,127.0.0.1,::1";

/// アプリケーションの起動設定です。
pub struct AppConfig {
    pub api_key: Arc<str>,
    pub upstream_url: Arc<str>,
    pub deep_api_key: Arc<str>,
    pub deep_upstream_url: Arc<str>,
    pub bind_addr: String,
    pub allowed_hosts: Vec<String>,
}

impl AppConfig {
    /// 環境変数および `.env` から設定を読み込みます。
    pub fn load() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let api_key = env::var("GROK_API_KEY")
            .map_err(|_| ConfigError::MissingVariable("GROK_API_KEY"))?
            .into();
        let upstream_url = env::var("GROK_UPSTREAM_URL")
            .map_err(|_| ConfigError::MissingVariable("GROK_UPSTREAM_URL"))?
            .into();
        let deep_api_key = env::var("GROK_DEEP_API_KEY")
            .map_err(|_| ConfigError::MissingVariable("GROK_DEEP_API_KEY"))?
            .into();
        let deep_upstream_url = env::var("GROK_DEEP_UPSTREAM_URL")
            .map_err(|_| ConfigError::MissingVariable("GROK_DEEP_UPSTREAM_URL"))?
            .into();
        let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_owned());
        let allowed_hosts = env::var("MCP_ALLOWED_HOSTS")
            .unwrap_or_else(|_| DEFAULT_ALLOWED_HOSTS.to_owned())
            .split(',')
            .map(str::trim)
            .filter(|host| !host.is_empty())
            .map(str::to_owned)
            .collect();

        Ok(Self {
            api_key,
            upstream_url,
            deep_api_key,
            deep_upstream_url,
            bind_addr,
            allowed_hosts,
        })
    }
}

/// 設定の読み込みに失敗した場合のエラーです。
#[derive(Debug)]
pub enum ConfigError {
    MissingVariable(&'static str),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingVariable(name) => {
                write!(formatter, "必須の環境変数 `{name}` が設定されていません")
            }
        }
    }
}

impl Error for ConfigError {}
