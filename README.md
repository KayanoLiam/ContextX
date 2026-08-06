# contextX

**日本語** | [简体中文](README.zh-CN.md) | [English](README.en.md)

contextXは、Grok 4.3による通常検索とGrok 4.20 Multi-Agent 0309による深度検索を提供するリモートMCPサーバーです。

MCPクライアントから受け取った検索クエリをOpenAI互換のResponses APIへストリーミング送信し、回答本文をMCPのツール結果として返します。上流ストリームを継続的に受信することで、検索処理中に発生するゲートウェイの待機タイムアウトを抑制します。利用者がモデルを変更することはできません。

**公開エンドポイント:** `https://mcp.twitter.monster/mcp`

## MCPクライアントへの追加

contextXはリモートMCPサーバーです。利用者側でバイナリをインストールしたり、APIキーを用意したりする必要はありません。

### Pi Agent

Pi本体にはMCP機能が組み込まれていないため、最初にMCPアダプターをインストールします。

```bash
pi install npm:pi-mcp-adapter
```

Piを再起動し、`~/.config/mcp/mcp.json`の`mcpServers`へ次の設定を追加します。既存のMCP設定がある場合は上書きせず、`contextX`の項目だけをマージしてください。

```json
{
  "mcpServers": {
    "contextX": {
      "url": "https://mcp.twitter.monster/mcp",
      "lifecycle": "lazy",
      "requestTimeoutMs": 360000
    }
  }
}
```

Piで接続を確認します。

```text
/mcp reconnect contextX
```

### Claude Code

```bash
claude mcp add --transport http contextX https://mcp.twitter.monster/mcp
```

Claude Code内の`/mcp`で接続状態を確認できます。

### Cursor

ユーザー全体の`~/.cursor/mcp.json`、またはプロジェクト内の`.cursor/mcp.json`へ追加します。

```json
{
  "mcpServers": {
    "contextX": {
      "url": "https://mcp.twitter.monster/mcp"
    }
  }
}
```

### その他のMCPクライアント

Streamable HTTP対応クライアントに、次のURLをリモートMCPサーバーとして登録してください。認証ヘッダーやトークンは不要です。

```text
https://mcp.twitter.monster/mcp
```

## セルフホスト

以下は、自分のAPIキーを使ってcontextXサーバーを運用する場合の手順です。

### ディレクトリ構成

```text
src/
├── main.rs    # エントリーポイント
├── config.rs  # 環境変数と起動設定
├── grok.rs    # Grok上流APIクライアント
├── mcp.rs     # MCPツールとサーバー情報
└── server.rs  # Streamable HTTPサーバー
```

### セットアップ

`.env.example`をコピーします。

```bash
cp .env.example .env
```

`.env`に上流APIキーとResponses APIの完全なエンドポイントを設定してください。実際の上流URLをソースコードや公開リポジトリへ記載しないでください。

```env
GROK_API_KEY=your-api-key
GROK_UPSTREAM_URL=https://your-upstream.example/v1/responses
GROK_DEEP_API_KEY=your-deep-search-api-key
GROK_DEEP_UPSTREAM_URL=https://your-deep-upstream.example/v1/responses
```

### 起動

```bash
cargo run
```

デフォルトでは次のURLで待ち受けます。

```text
MCP:    http://127.0.0.1:3000/mcp
Health: http://127.0.0.1:3000/health
```

### 環境変数

| 変数 | 必須 | 説明 |
|---|---:|---|
| `GROK_API_KEY` | はい | 通常検索上流APIのBearerトークン |
| `GROK_UPSTREAM_URL` | はい | 通常検索上流のResponses APIエンドポイント |
| `GROK_DEEP_API_KEY` | はい | 深度検索上流APIのBearerトークン |
| `GROK_DEEP_UPSTREAM_URL` | はい | 深度検索上流のResponses APIエンドポイント |
| `BIND_ADDR` | いいえ | サーバーの待受アドレス。デフォルトは`0.0.0.0:3000` |
| `MCP_ALLOWED_HOSTS` | いいえ | 受け入れるHostのカンマ区切りリスト |

公開時は`MCP_ALLOWED_HOSTS`に実際のドメインを追加してください。

```env
MCP_ALLOWED_HOSTS=localhost,127.0.0.1,::1,mcp.twitter.monster
```

## MCPツール

公開サービスでは、どちらのツールも認証や利用回数制限なしで利用できます。モデルや設定ファイルを変更する必要はなく、利用者が依頼内容によって検索方法を選択できます。

### `grok_search`

Grok 4.3で高速な通常検索を実行します。日常的な質問や簡単な情報確認に適しています。

入力例：

```json
{
  "query": "Rustの最新安定版について調べてください"
}
```

### `grok_deep_search`

Grok 4.20 Multi-Agent 0309で包括的な深度検索を実行します。通常検索より時間がかかるため、詳細な調査が必要な場合に使用してください。

入力例：

```json
{
  "query": "Rustの非同期ランタイムについて複数の情報源を比較し、詳細に調査してください"
}
```

MCPクライアントでは、「深度検索を使用してください」と明示することで深度検索ツールを選択できます。

## セキュリティ

- 上流APIキーはサーバー側だけに保存します。
- `.env`はGitの管理対象外です。
- 現在、MCP利用者に対する認証は行いません。