use std::{sync::Arc, time::Duration};

use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use reqwest::{Client, Response, header::CONTENT_TYPE};
use serde::Serialize;
use serde_json::Value;

/// 利用者からモデルを変更できないよう、上流モデルを固定します。
const MODEL: &str = "grok-4.3-fast";
/// 上流モデルへの固定指示は英語で記述し、利用者の入力言語で回答させます。
const INSTRUCTIONS: &str = "You are a web search assistant. Search the web and X when relevant. Provide an accurate and concise answer in the same language as the user's query. Include direct source URLs whenever available. Never fabricate sources, URLs, or claims.";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);
const ERROR_BODY_LIMIT: usize = 1_000;

/// Grok互換APIへのリクエストを担当するクライアントです。
#[derive(Clone)]
pub struct GrokClient {
    client: Client,
    api_key: Arc<str>,
    upstream_url: Arc<str>,
}

impl GrokClient {
    pub fn new(api_key: Arc<str>, upstream_url: Arc<str>) -> Result<Self, reqwest::Error> {
        let client = Client::builder().timeout(REQUEST_TIMEOUT).build()?;

        Ok(Self {
            client,
            api_key,
            upstream_url,
        })
    }

    /// Grokに検索クエリを送り、回答本文のみを返します。
    pub async fn search(&self, query: &str) -> Result<String, String> {
        let payload = ResponsesRequest {
            model: MODEL,
            instructions: INSTRUCTIONS,
            input: query,
            // 上流から継続的にイベントを受信し、ゲートウェイの待機タイムアウトを防ぎます。
            stream: true,
        };

        let response = self
            .client
            .post(self.upstream_url.as_ref())
            .bearer_auth(self.api_key.as_ref())
            .header("Accept", "text/event-stream")
            .json(&payload)
            .send()
            .await
            .map_err(|error| format!("Grok上流APIへのリクエストに失敗しました: {error}"))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .map_err(|error| format!("Grok上流APIのエラーを読み取れませんでした: {error}"))?;
            return Err(format!(
                "Grok上流APIがHTTP {status}を返しました: {}",
                truncate(&body, ERROR_BODY_LIMIT)
            ));
        }

        let is_event_stream = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("text/event-stream"));

        if is_event_stream {
            read_streaming_answer(response).await
        } else {
            // 互換APIがstream指定を無視して通常JSONを返す場合にも対応します。
            let body = response.text().await.map_err(|error| {
                format!("Grok上流APIのレスポンスを読み取れませんでした: {error}")
            })?;
            parse_non_streaming_answer(&body)
        }
    }
}

#[derive(Debug, Serialize)]
struct ResponsesRequest<'a> {
    model: &'static str,
    instructions: &'static str,
    input: &'a str,
    stream: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StreamControl {
    Continue,
    Completed,
}

async fn read_streaming_answer(response: Response) -> Result<String, String> {
    let mut events = response.bytes_stream().eventsource();
    let mut answer = String::new();

    while let Some(event) = events.next().await {
        let event =
            event.map_err(|error| format!("Grok上流APIのストリームが中断されました: {error}"))?;

        if process_stream_event(&event.event, &event.data, &mut answer)? == StreamControl::Completed
        {
            break;
        }
    }

    if answer.trim().is_empty() {
        Err("Grok上流APIのストリームから回答を取得できませんでした".to_owned())
    } else {
        Ok(answer)
    }
}

fn process_stream_event(
    event_name: &str,
    data: &str,
    answer: &mut String,
) -> Result<StreamControl, String> {
    let data = data.trim();
    if data.is_empty() {
        return Ok(StreamControl::Continue);
    }
    if data == "[DONE]" {
        return Ok(StreamControl::Completed);
    }

    let event: Value = match serde_json::from_str(data) {
        Ok(event) => event,
        Err(_) if event_name == "ping" => return Ok(StreamControl::Continue),
        Err(error) => {
            return Err(format!(
                "Grok上流APIが不正なストリームイベントを返しました: {error}"
            ));
        }
    };
    let event_type = event
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or(event_name);

    match event_type {
        "response.output_text.delta" => {
            if let Some(delta) = event.get("delta").and_then(Value::as_str) {
                answer.push_str(delta);
            }
            Ok(StreamControl::Continue)
        }
        "response.output_text.done" => {
            if let Some(text) = event.get("text").and_then(Value::as_str) {
                *answer = text.to_owned();
            }
            Ok(StreamControl::Continue)
        }
        "response.completed" => {
            let response = event.get("response").unwrap_or(&event);
            if let Some(text) = extract_answer(response) {
                *answer = text;
            }
            Ok(StreamControl::Completed)
        }
        "response.failed" | "response.incomplete" | "error" => Err(format!(
            "Grok上流APIの処理が完了しませんでした: {}",
            extract_stream_error(&event)
        )),
        _ => Ok(StreamControl::Continue),
    }
}

fn parse_non_streaming_answer(body: &str) -> Result<String, String> {
    let response: Value = serde_json::from_str(body)
        .map_err(|error| format!("Grok上流APIが不正なJSONを返しました: {error}"))?;

    extract_answer(&response).ok_or_else(|| {
        format!(
            "Grok上流APIのレスポンスから回答を取得できませんでした: {}",
            truncate(body, ERROR_BODY_LIMIT)
        )
    })
}

fn extract_answer(response: &Value) -> Option<String> {
    // 一部のOpenAI互換APIが返すトップレベルの補助フィールドにも対応します。
    if let Some(text) = response.get("output_text").and_then(Value::as_str)
        && !text.trim().is_empty()
    {
        return Some(text.to_owned());
    }

    let text = response
        .get("output")?
        .as_array()?
        .iter()
        .filter_map(|output| output.get("content").and_then(Value::as_array))
        .flatten()
        .filter(|part| {
            part.get("type")
                .and_then(Value::as_str)
                .is_none_or(|kind| kind == "output_text")
        })
        .filter_map(|part| part.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("\n");

    (!text.trim().is_empty()).then_some(text)
}

fn extract_stream_error(event: &Value) -> String {
    [
        "/response/error/message",
        "/response/incomplete_details/reason",
        "/error/message",
        "/message",
    ]
    .into_iter()
    .find_map(|pointer| event.pointer(pointer).and_then(Value::as_str))
    .unwrap_or("詳細不明")
    .to_owned()
}

fn truncate(text: &str, max_chars: usize) -> String {
    let mut chars = text.chars();
    let shortened: String = chars.by_ref().take(max_chars).collect();

    if chars.next().is_some() {
        format!("{shortened}…")
    } else {
        shortened
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{INSTRUCTIONS, StreamControl, extract_answer, process_stream_event, truncate};

    #[test]
    fn upstream_instructions_are_english() {
        assert!(INSTRUCTIONS.is_ascii());
        assert!(INSTRUCTIONS.contains("same language as the user's query"));
    }

    #[test]
    fn extracts_output_text() {
        let response = json!({
            "output": [{
                "type": "message",
                "content": [{
                    "type": "output_text",
                    "text": "検索結果です"
                }]
            }]
        });

        assert_eq!(extract_answer(&response).as_deref(), Some("検索結果です"));
    }

    #[test]
    fn combines_multiple_output_text_parts() {
        let response = json!({
            "output": [
                {
                    "type": "message",
                    "content": [{ "type": "output_text", "text": "1行目" }]
                },
                {
                    "type": "message",
                    "content": [{ "type": "output_text", "text": "2行目" }]
                }
            ]
        });

        assert_eq!(extract_answer(&response).as_deref(), Some("1行目\n2行目"));
    }

    #[test]
    fn extracts_top_level_output_text() {
        let response = json!({ "output_text": "検索結果です" });

        assert_eq!(extract_answer(&response).as_deref(), Some("検索結果です"));
    }

    #[test]
    fn appends_streaming_deltas() {
        let mut answer = String::new();

        let first = process_stream_event(
            "response.output_text.delta",
            r#"{"type":"response.output_text.delta","delta":"検索"}"#,
            &mut answer,
        );
        let second = process_stream_event(
            "response.output_text.delta",
            r#"{"type":"response.output_text.delta","delta":"結果"}"#,
            &mut answer,
        );

        assert_eq!(first, Ok(StreamControl::Continue));
        assert_eq!(second, Ok(StreamControl::Continue));
        assert_eq!(answer, "検索結果");
    }

    #[test]
    fn uses_completed_response_as_final_answer() {
        let mut answer = "途中".to_owned();
        let data = json!({
            "type": "response.completed",
            "response": {
                "output": [{
                    "type": "message",
                    "content": [{ "type": "output_text", "text": "最終回答" }]
                }]
            }
        })
        .to_string();

        let result = process_stream_event("response.completed", &data, &mut answer);

        assert_eq!(result, Ok(StreamControl::Completed));
        assert_eq!(answer, "最終回答");
    }

    #[test]
    fn truncates_by_character_count() {
        assert_eq!(truncate("あいうえお", 3), "あいう…");
        assert_eq!(truncate("あいう", 3), "あいう");
    }
}
