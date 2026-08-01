# contextX

**日本語** | [简体中文](README.zh-CN.md)

contextXは、Grok 4.3 Fastを利用してウェブ検索を行うリモートMCPサーバーです。

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
| `GROK_API_KEY` | はい | 上流APIのBearerトークン |
| `GROK_UPSTREAM_URL` | はい | OpenAI互換のResponses APIエンドポイント。ソースコード内に既定値はありません |
| `BIND_ADDR` | いいえ | サーバーの待受アドレス。デフォルトは`0.0.0.0:3000` |
| `MCP_ALLOWED_HOSTS` | いいえ | 受け入れるHostのカンマ区切りリスト |

公開時は`MCP_ALLOWED_HOSTS`に実際のドメインを追加してください。

```env
MCP_ALLOWED_HOSTS=localhost,127.0.0.1,::1,mcp.twitter.monster
```

## MCPツール

### `grok_search`

Grok 4.3 Fastでウェブ検索し、質問に回答します。

入力例：

```json
{
  "query": "Rustの最新安定版について調べてください"
}
```

## セキュリティ

- 上流APIキーはサーバー側だけに保存します。
- `.env`はGitの管理対象外です。
- 現在、MCP利用者に対する認証は行いません。

---

## 友好リンク

[![LINUX DO](https://img.shields.io/badge/LINUX%20DO-FFB003.svg?style=for-the-badge&logo=data:image/svg%2bxml;base64,DQo8c3ZnIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgd2lkdGg9IjEwMCIgaGVpZ2h0PSIxMDAiPjxwYXRoIGQ9Ik00Ni44Mi0uMDU1aDYuMjVxMjMuOTY5IDIuMDYyIDM4IDIxLjQyNmM1LjI1OCA3LjY3NiA4LjIxNSAxNi4xNTYgOC44NzUgMjUuNDV2Ni4yNXEtMi4wNjQgMjMuOTY4LTIxLjQzIDM4LTExLjUxMiA3Ljg4NS0yNS40NDUgOC44NzRoLTYuMjVxLTIzLjk3LTIuMDY0LTM4LjAwNC0yMS40M1EuOTcxIDY3LjA1Ni0uMDU0IDUzLjE4di02LjQ3M0MxLjM2MiAzMC43ODEgOC41MDMgMTguMTQ4IDIxLjM3IDguODE3IDI5LjA0NyAzLjU2MiAzNy41MjcuNjA0IDQ2LjgyMS0uMDU2IiBzdHlsZT0ic3Ryb2tlOm5vbmU7ZmlsbC1ydWxlOmV2ZW5vZGQ7ZmlsbDojZWNlY2VjO2ZpbGwtb3BhY2l0eToxIi8+PHBhdGggZD0iTTQ3LjI2NiAyLjk1N3EyMi41My0uNjUgMzcuNzc3IDE1LjczOGE0OS43IDQ5LjcgMCAwIDEgNi44NjcgMTAuMTU3cS00MS45NjQuMjIyLTgzLjkzIDAgOS43NS0xOC42MTYgMzAuMDI0LTI0LjM4N2E2MSA2MSAwIDAgMSA5LjI2Mi0xLjUwOCIgc3R5bGU9InN0cm9rZTpub25lO2ZpbGwtcnVsZTpldmVub2RkO2ZpbGw6IzE5MTkxOTtmaWxsLW9wYWNpdHk6MSIvPjxwYXRoIGQ9Ik03Ljk4IDcwLjkyNmMyNy45NzctLjAzNSA1NS45NTQgMCA4My45My4xMTNRODMuNDI2IDg3LjQ3MyA2Ni4xMyA5NC4wODZxLTE4LjgxIDYuNTQ0LTM2LjgzMi0xLjg5OC0xNC4yMDMtNy4wOS0yMS4zMTctMjEuMjYyIiBzdHlsZT0ic3Ryb2tlOm5vbmU7ZmlsbC1ydWxlOmV2ZW5vZGQ7ZmlsbDojZjlhZjAwO2ZpbGwtb3BhY2l0eToxIi8+PC9zdmc+)](https://linux.do/)
