# contextX

[日本語](README.md) | [简体中文](README.zh-CN.md) | **English**

contextX is a remote MCP server that provides normal search powered by Grok 4.3 and deep search powered by Grok 4.20 Multi-Agent 0309.

It streams search queries received from the MCP client to the upstream OpenAI-compatible Responses API, and returns the generated content as the MCP tool result. Continuously receiving the upstream stream reduces gateway wait timeouts that may occur during the search process. Users cannot change the models.

**Public Endpoint:** `https://mcp.twitter.monster/mcp`

## Adding to MCP Clients

contextX is a remote MCP server. Users do not need to install any binaries or provide API keys.

### Pi Agent

Since Pi does not have built-in MCP functionality, install the MCP adapter first:

```bash
pi install npm:pi-mcp-adapter
```

Restart Pi, and add the following configuration to `mcpServers` in `~/.config/mcp/mcp.json`. If you already have existing MCP configurations, please merge only the `contextX` item and do not overwrite the entire file.

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

Check the connection in Pi:

```text
/mcp reconnect contextX
```

### Claude Code

```bash
claude mcp add --transport http contextX https://mcp.twitter.monster/mcp
```

You can verify the connection status via `/mcp` in Claude Code.

### Cursor

Add the configuration to the user-level `~/.cursor/mcp.json` or project-level `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "contextX": {
      "url": "https://mcp.twitter.monster/mcp"
    }
  }
}
```

### Other MCP Clients

For clients supporting Streamable HTTP, simply register the following URL as a remote MCP server. No authentication headers or tokens are required:

```text
https://mcp.twitter.monster/mcp
```

## Self-Hosting

The following instructions are for users who want to host the contextX server using their own API keys.

### Directory Structure

```text
src/
├── main.rs    # Entry point
├── config.rs  # Environment variables and startup configuration
├── grok.rs    # Grok upstream API client
├── mcp.rs     # MCP tools and server info
└── server.rs  # Streamable HTTP server
```

### Setup

Copy the example environment file:

```bash
cp .env.example .env
```

Set your upstream API keys and the full Responses API endpoints in `.env`. Do not write actual upstream URLs into the source code or public repositories:

```env
GROK_API_KEY=your-api-key
GROK_UPSTREAM_URL=https://your-upstream.example/v1/responses
GROK_DEEP_API_KEY=your-deep-search-api-key
GROK_DEEP_UPSTREAM_URL=https://your-deep-upstream.example/v1/responses
```

### Start

```bash
cargo run
```

The default listening addresses are:

```text
MCP:    http://127.0.0.1:3000/mcp
Health: http://127.0.0.1:3000/health
```

### Environment Variables

| Variable | Required | Description |
|---|---:|---|
| `GROK_API_KEY` | Yes | Bearer Token for the normal search upstream API |
| `GROK_UPSTREAM_URL` | Yes | Responses API endpoint for normal search upstream |
| `GROK_DEEP_API_KEY` | Yes | Bearer Token for the deep search upstream API |
| `GROK_DEEP_UPSTREAM_URL` | Yes | Responses API endpoint for deep search upstream |
| `BIND_ADDR` | No | Server listen address, defaults to `0.0.0.0:3000` |
| `MCP_ALLOWED_HOSTS` | No | Comma-separated list of allowed Hosts |

When deploying publicly, add your actual domain to `MCP_ALLOWED_HOSTS`:

```env
MCP_ALLOWED_HOSTS=localhost,127.0.0.1,::1,mcp.twitter.monster
```

## MCP Tools

Both tools in the public service require no authentication and have no rate limits. Users do not need to modify models or configuration files, but simply choose normal or deep search based on their request.

### `grok_search`

Executes high-speed normal search using Grok 4.3, suitable for daily questions and simple information retrieval.

Example input:

```json
{
  "query": "Find out the latest stable version of Rust"
}
```

### `grok_deep_search`

Executes comprehensive deep search using Grok 4.20 Multi-Agent 0309. It takes longer than normal search, so use it when detailed investigation and cross-referencing multiple sources are needed.

Example input:

```json
{
  "query": "Deep dive into Rust's async runtimes and compare multiple sources"
}
```

In your MCP client, explicitly requesting "please use deep search" will prompt the agent to select the deep search tool.

## Security Notes

- Upstream API keys are stored on the server side only.
- `.env` is excluded from Git tracking.
- Currently, no authentication is required for MCP users.