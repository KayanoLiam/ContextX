use rmcp::{
    ErrorData, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::grok::GrokClient;

/// MCPツールに渡す検索パラメーターです。
#[derive(Debug, Deserialize, JsonSchema)]
struct SearchParams {
    /// Grokでウェブ検索し、回答してほしい質問です。
    query: String,
}

/// contextXのMCPリクエストを処理します。
#[derive(Clone)]
pub struct ContextXServer {
    grok_client: GrokClient,
    #[expect(dead_code, reason = "tool_handlerマクロがこのルーターを参照します")]
    tool_router: ToolRouter<Self>,
}

impl ContextXServer {
    pub fn new(grok_client: GrokClient) -> Self {
        Self {
            grok_client,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl ContextXServer {
    #[tool(description = "Grok 4.3 Fastでウェブ検索し、質問に回答します")]
    async fn grok_search(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let query = params.query.trim();
        if query.is_empty() {
            return Err(ErrorData::invalid_params(
                "`query`には空でない文字列を指定してください",
                None,
            ));
        }

        match self.grok_client.search(query).await {
            Ok(answer) => Ok(CallToolResult::success(vec![ContentBlock::text(answer)])),
            Err(message) => Ok(CallToolResult::error(vec![ContentBlock::text(message)])),
        }
    }

    #[tool(
        description = "Grok 4.20 Multi-Agent XHighで包括的な深度検索を実行します。利用者が深度検索を明示的に要求した場合に使用します"
    )]
    async fn grok_deep_search(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let query = params.query.trim();
        if query.is_empty() {
            return Err(ErrorData::invalid_params(
                "`query`には空でない文字列を指定してください",
                None,
            ));
        }

        match self.grok_client.deep_search(query).await {
            Ok(answer) => Ok(CallToolResult::success(vec![ContentBlock::text(answer)])),
            Err(message) => Ok(CallToolResult::error(vec![ContentBlock::text(message)])),
        }
    }
}

#[tool_handler]
impl ServerHandler for ContextXServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new("contextX", env!("CARGO_PKG_VERSION"))
                    .with_title("contextX Grok検索"),
            )
            .with_instructions(
                "通常検索にはGrok 4.3 Fastを使用します。利用者が深度検索を明示的に要求した場合は、Grok 4.20 Multi-Agent XHighによる深度検索を使用します。",
            )
    }
}
