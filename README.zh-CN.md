# SenAgentOS

**SenAgentOS** 是一款新一代自主式 AI Agent 操作系统，完全使用 Rust 构建。它提供了一个高性能、可扩展的运行时，用于构建、部署和编排 AI Agent，连接各种消息平台、生产力工具乃至硬件外设。

SenAgentOS 起源于对 `cc-typescript-src`（驱动 cc 类 AI Agent 的 TypeScript 代码库）进行的 Rust 重写与性能升级。Rust 版本保留了原始模块化、Trait 驱动的架构设计，同时带来了更优越的性能、内存安全保证和零成本抽象。项目中的几乎所有模块都能在 `cc-typescript-src` 中找到对应实现，均以惯用 Rust 方式重新编写，全面支持 async、精细化内存控制以及原生跨平台编译。

---

## 核心特性

### Agent 智能

- **多 Agent 编排** — 基于 Supervisor 的 Agent 监管机制，支持健康监控、自动重启和优雅关闭。Coordinator 提供分布式锁、屏障同步和投票共识，支持复杂的多 Agent 工作流。
- **自我进化** — 内置强化学习、经验回放、自我反思和自主技能创建。Agent 能够从自身历史中学习并持续改进。
- **工具生态** — 100+ 内置工具，覆盖 Shell 命令、文件操作、网页浏览、Git 操作、Notion、JIRA、Google Workspace、Microsoft 365 等生产力应用，以及图像生成等能力。
- **查询分类** — 根据查询意图动态路由模型，简单任务用轻量模型，复杂推理交给更强大的模型。
- **上下文管理** — Token 预算管理、上下文压缩、循环检测、智能记忆加载，确保始终在上下文窗口限制内运行。

### 集成能力

- **45+ 消息渠道** — Telegram、Discord、Slack、Matrix（支持 E2EE）、WhatsApp（原生 + Web）、Signal、IRC、Nostr、Mastodon、Bluesky、Reddit、Twitter/X 等。
- **中国平台生态** —  Lark/飞书、钉钉、企业微信、QQ、墨千、Line/Linq。
- **18+ LLM Provider** — OpenRouter、OpenAI、Anthropic、Google Gemini、Azure OpenAI、AWS Bedrock、Ollama、Groq、Mistral、Deepseek、Cohere、Together AI、Fireworks AI、Z.AI GLM、Claude Code、GitHub Copilot 等。
- **智能模型路由** — 基于查询类型自动选择最适合的模型。

### 记忆与知识

- **多记忆后端** — SQLite（结构化存储）、Markdown（文件知识库）、Qdrant（向量搜索）、知识图谱（关系记忆）、共享黑板（多 Agent 协调）。
- **RAG 管道** — 基于 OpenAI 兼容 Embeddings 的检索增强生成，支持可配置的文本分块策略和重要性评分驱动的记忆整合。
- **GDPR 就绪** — 支持数据导出、记忆清理和自动维护。

### 安全性

- **全面沙箱** — Docker、Firejail、Bubblewrap、Landlock（Linux 内核）和 Seatbelt（macOS）等多种进程隔离方案。
- **访问控制** — 基于角色的访问控制（RBAC）、策略化授权、Nevis 企业级 IAM 集成。
- **密钥管理** — ChaCha20-Poly1305 加密存储、HMAC Webhook 签名验证。
- **运营安全** — 紧急停止开关（全局终止、网络隔离、域名封禁、工具冻结），支持快速事件响应。
- **提示词注入防护** — 泄露检测、污染追踪、域名白名单。

### 硬件与 IoT

- **板级支持** — STM32 Nucleo-F401RE、树莓派 GPIO、Arduino Uno（通过串口桥接）。
- **外设抽象层** —  Trait 驱动的硬件 API，便于添加新的开发板支持。
- **IoT 协议** — MQTT 代理集成，支持传感器和执行器的远程控制。

### 可扩展性

- **插件系统** — 基于 Extism 的 WASM 运行时，支持运行时动态扩展功能。
- **A2A 协议** — 标准化的 Agent 间通信协议，兼容多供应商 Agent 生态。
- **MCP（Model Context Protocol）** — 内置 MCP 工具桥接。
- **Composio 集成** — 直接访问 Composio 工具注册表。

### 可观测性

- **Prometheus 指标** — 内置指标端点。
- **OpenTelemetry** — 分布式追踪支持。
- **健康检查** — `/health` 和 `/metrics` 端点供监控使用。

---

## 架构设计

SenAgentOS 采用 Trait 驱动的模块化架构。Provider、Channel、Tool、Memory 后端、外设、运行时适配器等所有核心组件都通过定义明确的 Rust Trait 实现。添加新的 Provider、Channel 或 Tool 只需实现相应 Trait 并在工厂模块中注册即可。

### 关键设计理念

- **Rust 优先** — 核心 Agent 运行时 100% Rust，支持 Linux、macOS、Windows 和树莓派等嵌入式平台。
- **Trait 驱动** — 所有扩展点都是 Trait，扩展系统只需实现 Trait + 注册，无需修改核心代码。
- **记忆优先** — 一流的记忆管理，多后端支持，向量搜索，重要性评分和自动整合。
- **安全内建** — 沙箱、RBAC、密钥存储、提示词注入防御均为内置功能，非后期叠加。
- **自主进化** — Agent 能够自主创建新技能、优化提示词并根据反馈持续改进。

---

## 从源码构建

### 环境要求

- **Rust 1.87+**（Edition 2024）
- **Node.js 20+**（用于 Web 前端，Windows 可用 `winget install OpenJS.NodeJS.LTS` 安装）
- **CMake**（部分原生依赖需要）

#### 1. Visual Studio Build Tools（仅 Windows）

在 Windows 上，原生依赖需要 MSVC 工具链和 Windows SDK：

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

安装过程中（或通过 Visual Studio Installer）勾选 **"Desktop development with C++"** 工作负载。

#### 2. Rust 工具链

**Linux / macOS：**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
```

**Windows：**

```powershell
winget install Rustlang.Rustup
# 安装完成后打开新终端窗口，运行：
rustup default stable
```

#### 3. 验证环境

```powershell
rustc --version
cargo --version
```

### 克隆仓库

```bash
git clone https://github.com/senweaver/SenAgentOS.git
cd SenAgentOS
```

### 仅构建后端

如果你只需要 CLI Agent 而不需要 Web 界面：

```bash
cargo build --release
```

二进制文件位于 `target/release/senagentos`。

### 前后端完整构建

构建脚本会自动将 React 前端打包进二进制文件，一份 `senagentos` 可执行文件同时提供 CLI 和 Web Dashboard。

```bash
cargo build --release --features default
```

> **注意：**首次构建需要编译整个 Rust 后端（视机器配置不同，需 5–20 分钟）。后续增量编译速度会快很多。

### 前端开发模式

如果你在同时进行前端开发，可以单独启动 Vite 开发服务器（支持热更新），让前端连接已有的 Gateway：

```bash
# 终端 1：启动 Gateway（提供 API）
./target/release/senagentos gateway start

# 终端 2：启动前端开发服务器（端口 5173）
cd web
npm install
npm run dev
```

前端开发服务器会自动将 API 请求代理到 `http://localhost:42617`。

### 前端独立构建

仅构建前端用于部署：

```bash
cd web
npm install
npm run build
```

产物位于 `web/dist/`。

---

## 配置

SenAgentOS 使用 `config.toml` 配置文件。复制示例文件后修改：

```bash
cp .env.example .env
# 编辑 .env 或创建 ~/.senagent/config.toml
```

### 主要环境变量

| 变量 | 说明 | 默认值 |
|---|---|---|
| `PROVIDER` | LLM Provider 名称 | `openrouter` |
| `SENWEAVER_API_KEY` | SenWeaver API Key | — |
| `SENWEAVER_BASE_URL` | SenWeaver API 地址 | `https://api3.senweaver.com/v1` |
| `OPENAI_API_KEY` | OpenAI API Key | — |
| `ANTHROPIC_API_KEY` | Anthropic API Key | — |
| `SENAGENTOS_WORKSPACE` | 工作目录 | `~/.senagent/workspace` |

### 使用 SenWeaver 快速启动（推荐）

SenWeaver 提供聚合多 Provider 的统一 API：

```bash
# 设置环境变量
export SENWEAVER_API_KEY="your-api-key"
export SENWEAVER_BASE_URL="https://api3.senweaver.com/v1"
export PROVIDER="senweaver"

# 交互式引导配置
./target/release/senagentos onboard --quick

# 或直接启动带 Web Dashboard 的 Gateway
./target/release/senagentos gateway start
```

### 使用 OpenRouter 快速启动

```bash
export OPENROUTER_API_KEY="your-api-key"
export PROVIDER="openrouter"
./target/release/senagentos agent
```

### 配置文件

生产环境建议使用 `~/.senagent/config.toml`：

```toml
[provider]
default = "openrouter"

[provider.openrouter]
api_key = "sk-..."

[agent]
max_iterations = 100
tools = ["file_read", "file_write", "shell", "browser"]

[memory]
backend = "sqlite"

[[channel]]
type = "telegram"
bot_token = "..."

[[channel.telegram]]
streaming = true
```

运行 `./target/release/senagentos config --help` 查看完整配置参考。

---

## 运行

### CLI Agent 模式

在终端中交互式运行 Agent：

```bash
./senagentos agent
```

直接传入消息：

```bash
./senagentos agent --message "What's the weather in Tokyo?"
```

### Gateway 模式（Web Dashboard）

启动 HTTP/WebSocket Gateway：

```bash
./senagentos gateway start
```

Gateway 提供以下服务：

- **Web Dashboard** — 在浏览器打开 `http://localhost:42617/`
- **REST API** — `http://localhost:42617/api/*`
- **WebSocket 聊天** — `ws://localhost:42617/ws/chat`
- **健康检查** — `http://localhost:42617/health`
- **Prometheus 指标** — `http://localhost:42617/metrics`

### 渠道机器人

运行独立的渠道 Bot：

```bash
./senagentos channel telegram --token "your-bot-token"
./senagentos channel discord --token "your-bot-token"
./senagentos channel slack --token "xoxb-..."
```

### 自检

验证配置是否正确：

```bash
./senagentos self-test
```

---

## Web Dashboard

Web Dashboard 为管理 Agent 提供了完整的图形界面：

| 页面 | 说明 |
|---|---|
| **Agent Chat** | 流式响应的交互式对话界面 |
| **Dashboard** | 系统状态、资源使用、活跃 Agent |
| **Tools** | 浏览和管理可用工具 |
| **Cron** | 定时任务的创建与管理 |
| **Memory** | 浏览 Agent 的记忆存储 |
| **Config** | 在线编辑配置文件 |
| **Logs** | 实时日志查看器，支持过滤 |
| **Doctor** | 健康诊断与问题排查 |
| **Integrations** | 管理渠道连接 |
| **Skills** | 查看和管理 Agent 技能 |
| **Hardware** | 监控和控制硬件外设 |
| **Canvas** | Agent 共享可视化工作空间 |
| **Settings** | 用户偏好设置 |

技术栈：React 19、React Router 7、TypeScript、Tailwind CSS 4、Vite 6、Lucide React 图标。

---

## 扩展集成

SenAgentOS 的主要扩展点：

- **Provider** — 在 providers 模块中添加新模块并注册工厂来支持新的 LLM Provider。
- **Channel** — 在 channels 模块中创建新模块并实现 Channel Trait 来添加新的消息平台。
- **Tool** — 在 tools 模块中创建新模块并实现 Tool Trait 来扩展工具能力。
- **Memory 后端** — 在 memory 模块中创建新模块并实现 Memory Trait 来添加新的记忆存储。

---

## 安全性

生产部署前请阅读 [安全文档](docs/security/)。重点关注：

- **沙箱** — 至少启用一种沙箱后端（`sandbox-docker`、`sandbox-firejail` 等）来隔离工具执行。
- **RBAC** — 为工具和渠道配置基于角色的访问控制。
- **配对认证** — Gateway 访问建议使用设备配对而非开放访问。
- **密钥安全** — 勿将 API Key 提交到代码仓库，使用密钥存储或环境变量。
- **紧急停止** — 熟悉紧急停止命令，以便快速响应安全事件。

### 沙箱后端支持情况

| 后端 | Linux | macOS | Windows |
|---|---|---|---|
| Docker | ✓ | — | — |
| Firejail | ✓ | — | — |
| Bubblewrap | ✓ | — | — |
| Landlock | ✓ | — | — |
| Seatbelt | — | ✓ | — |

---

## 项目状态

SenAgentOS 正在活跃开发中，核心 Agent、Gateway 和主要集成均已可用。部分功能仍在完善：

- **技能进化** — 自主技能创建已可运行，技能市场属于规划中功能。
- **硬件支持** — STM32 和树莓派板级支持已可用，Arduino 桥接稳定。
- **插件系统** — WASM 插件运行时已集成，SDK 仍在开发中。
- **A2A 协议** — 核心实现已完成，生态落地处于早期阶段。

---

## 贡献

欢迎提交贡献。提交 PR 前请阅读[贡献指南](docs/contributing/)。

- 请在非 `master` 分支上工作，PR 提交到 `master`。
- 遵循约定式提交规范（`feat:`、`fix:`、`docs:` 等）。
- 提交前请运行检查：`cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo test`。

---

## 许可证

MIT 许可证，详见 [LICENSE](LICENSE)。
