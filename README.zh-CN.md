# contextX

[日本語](README.md) | **简体中文** | [English](README.en.md)

contextX 是一个同时提供 Grok 4.3 普通搜索和 Grok 4.20 Multi-Agent 0309 深度搜索的远程 MCP 服务。

服务通过 OpenAI 兼容的 Responses API，以流式方式将 MCP 客户端提交的查询发送给上游，并将最终回答作为 MCP 工具结果返回。持续接收上游数据可以减少搜索过程中出现的网关等待超时。用户无法自行更改模型。

**公共服务地址：** `https://mcp.twitter.monster/mcp`

## 添加到 MCP 客户端

contextX 是远程 MCP 服务。普通用户不需要安装二进制文件，也不需要提供 API Key。

### Pi Agent

Pi 本身不内置 MCP 功能，需要先安装 MCP Adapter：

```bash
pi install npm:pi-mcp-adapter
```

重启 Pi，然后将以下内容添加到 `~/.config/mcp/mcp.json` 的 `mcpServers` 中。如果已有其他 MCP 配置，请只合并 `contextX` 项目，不要覆盖整个文件。

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

在 Pi 中检查连接：

```text
/mcp reconnect contextX
```

### Claude Code

```bash
claude mcp add --transport http contextX https://mcp.twitter.monster/mcp
```

可以通过 Claude Code 中的 `/mcp` 检查连接状态。

### Cursor

将配置添加到用户级 `~/.cursor/mcp.json`，或者项目内的 `.cursor/mcp.json`：

```json
{
  "mcpServers": {
    "contextX": {
      "url": "https://mcp.twitter.monster/mcp"
    }
  }
}
```

### 其他 MCP 客户端

在支持 Streamable HTTP 的客户端中，将以下地址注册为远程 MCP 服务即可，不需要认证请求头或 Token：

```text
https://mcp.twitter.monster/mcp
```

## 自行部署

以下内容适用于希望使用自己的 API Key 部署 contextX 服务的用户。

### 目录结构

```text
src/
├── main.rs    # 程序入口
├── config.rs  # 环境变量和启动配置
├── grok.rs    # Grok 上游 API 客户端
├── mcp.rs     # MCP 工具和服务信息
└── server.rs  # Streamable HTTP 服务
```

### 配置

复制环境变量示例文件：

```bash
cp .env.example .env
```

在 `.env` 中设置上游 API Key 和完整的 Responses API 地址。请勿将真实上游地址写入源代码或公开仓库：

```env
GROK_API_KEY=your-api-key
GROK_UPSTREAM_URL=https://your-upstream.example/v1/responses
GROK_DEEP_API_KEY=your-deep-search-api-key
GROK_DEEP_UPSTREAM_URL=https://your-deep-upstream.example/v1/responses
```

### 启动

```bash
cargo run
```

默认监听地址：

```text
MCP:    http://127.0.0.1:3000/mcp
Health: http://127.0.0.1:3000/health
```

### 环境变量

| 变量 | 必填 | 说明 |
|---|---:|---|
| `GROK_API_KEY` | 是 | 普通搜索上游 API 的 Bearer Token |
| `GROK_UPSTREAM_URL` | 是 | 普通搜索上游的 Responses API 地址 |
| `GROK_DEEP_API_KEY` | 是 | 深度搜索上游 API 的 Bearer Token |
| `GROK_DEEP_UPSTREAM_URL` | 是 | 深度搜索上游的 Responses API 地址 |
| `BIND_ADDR` | 否 | 服务监听地址，默认为 `0.0.0.0:3000` |
| `MCP_ALLOWED_HOSTS` | 否 | 允许访问的 Host，使用英文逗号分隔 |

公开部署时，请将实际域名加入 `MCP_ALLOWED_HOSTS`：

```env
MCP_ALLOWED_HOSTS=localhost,127.0.0.1,::1,mcp.twitter.monster
```

## MCP 工具

公共服务中的两个工具均无需认证，并且不限制调用次数。用户不需要修改模型或配置文件，只需根据需求选择普通搜索或深度搜索。

### `grok_search`

使用 Grok 4.3 执行高速普通搜索，适合日常问题和简单信息查询。

输入示例：

```json
{
  "query": "查询 Rust 当前最新稳定版本"
}
```

### `grok_deep_search`

使用 Grok 4.20 Multi-Agent 0309 执行全面的深度搜索。它比普通搜索耗时更长，适合需要详细调查和多来源对比的问题。

输入示例：

```json
{
  "query": "对 Rust 异步运行时进行深度调查，并比较多个信息来源"
}
```

在 MCP 客户端中明确说明“请使用深度搜索”，即可让 Agent 选择深度搜索工具。

## 安全说明

- 上游 API Key 仅保存在服务器端。
- `.env` 已被 Git 忽略。
- 当前不对 MCP 用户进行身份认证。

---

## 友情链接

[![LINUX DO](https://img.shields.io/badge/LINUX%20DO-FFB003.svg?style=for-the-badge&logo=data:image/svg%2bxml;base64,DQo8c3ZnIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgd2lkdGg9IjEwMCIgaGVpZ2h0PSIxMDAiPjxwYXRoIGQ9Ik00Ni44Mi0uMDU1aDYuMjVxMjMuOTY5IDIuMDYyIDM4IDIxLjQyNmM1LjI1OCA3LjY3NiA4LjIxNSAxNi4xNTYgOC44NzUgMjUuNDV2Ni4yNXEtMi4wNjQgMjMuOTY4LTIxLjQzIDM4LTExLjUxMiA3Ljg4NS0yNS40NDUgOC44NzRoLTYuMjVxLTIzLjk3LTIuMDY0LTM4LjAwNC0yMS40M1EuOTcxIDY3LjA1Ni0uMDU0IDUzLjE4di02LjQ3M0MxLjM2MiAzMC43ODEgOC41MDMgMTguMTQ4IDIxLjM3IDguODE3IDI5LjA0NyAzLjU2MiAzNy41MjcuNjA0IDQ2LjgyMS0uMDU2IiBzdHlsZT0ic3Ryb2tlOm5vbmU7ZmlsbC1ydWxlOmV2ZW5vZGQ7ZmlsbDojZWNlY2VjO2ZpbGwtb3BhY2l0eToxIi8+PHBhdGggZD0iTTQ3LjI2NiAyLjk1N3EyMi41My0uNjUgMzcuNzc3IDE1LjczOGE0OS43IDQ5LjcgMCAwIDEgNi44NjcgMTAuMTU3cS00MS45NjQuMjIyLTgzLjkzIDAgOS43NS0xOC42MTYgMzAuMDI0LTI0LjM4N2E2MSA2MSAwIDAgMSA5LjI2Mi0xLjUwOCIgc3R5bGU9InN0cm9rZTpub25lO2ZpbGwtcnVsZTpldmVub2RkO2ZpbGw6IzE5MTkxOTtmaWxsLW9wYWNpdHk6MSIvPjxwYXRoIGQ9Ik03Ljk4IDcwLjkyNmMyNy45NzctLjAzNSA1NS45NTQgMCA4My45My4xMTNRODMuNDI2IDg3LjQ3MyA2Ni4xMyA5NC4wODZxLTE4LjgxIDYuNTQ0LTM2LjgzMi0xLjg5OC0xNC4yMDMtNy4wOS0yMS4zMTctMjEuMjYyIiBzdHlsZT0ic3Ryb2tlOm5vbmU7ZmlsbC1ydWxlOmV2ZW5vZGQ7ZmlsbDojZjlhZjAwO2ZpbGwtb3BhY2l0eToxIi8+PC9zdmc+)](https://linux.do/)
