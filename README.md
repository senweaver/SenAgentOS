# SenAgentOS

**SenAgentOS** is a next-generation autonomous AI agent operating system built entirely in Rust. It provides a high-performance, extensible runtime for building, deploying, and orchestrating AI agents that connect to the world — from messaging platforms and productivity tools to hardware peripherals.

SenAgentOS originated as a Rust rewrite and performance upgrade of `cc-typescript-src` — the TypeScript codebase powering cc-class AI agents. The Rust port preserves the original modular, trait-driven architecture while delivering superior performance, memory safety, and zero-cost abstractions. Nearly every module in this codebase mirrors a counterpart in `cc-typescript-src`, reimplemented in idiomatic Rust with full async support, tighter memory control, and native cross-platform compilation.

---

## Key Features

### Agent Intelligence
- **Multi-Agent Orchestration** — Supervisor-based agent supervision with health monitoring, auto-restart, and graceful shutdown. Coordinator provides distributed locks, barrier synchronization, and voting-based consensus for multi-agent workflows.
- **Self-Evolution** — Built-in reinforcement learning, experience replay, self-reflection, and autonomous skill creation. Agents learn from their own history and improve over time.
- **Tool Ecosystem** — 100+ built-in tools covering shell commands, file operations, web browsing, Git operations, productivity apps (Notion, JIRA, Google Workspace, Microsoft 365), image generation, and more.
- **Query Classification** — Dynamic model routing based on query intent. Cheap models for simple tasks, powerful models for complex reasoning.
- **Context Management** — Token budget management, context compression, loop detection, and intelligent memory loading to stay within context windows.

### Integrations
- **45+ Messaging Channels** — Telegram, Discord, Slack, Matrix (with E2EE), WhatsApp (native + web), Signal, IRC, Nostr, Mastodon, Bluesky, Reddit, Twitter/X, and more.
- **Chinese Platform Suite** — Lark/Feishu, DingTalk, WeCom, QQ, MoChat, and Line/Linq.
- **18+ LLM Providers** — OpenRouter, OpenAI, Anthropic, Google Gemini, Azure OpenAI, AWS Bedrock, Ollama, Groq, Mistral, Deepseek, Cohere, Together AI, Fireworks AI, Z.AI GLM, Claude Code, GitHub Copilot, and more.
- **Model Routing** — Intelligent query-based routing to the right model for each task.

### Memory & Knowledge
- **Multiple Memory Backends** — SQLite for structured storage, Markdown for file-based knowledge, Qdrant for vector search, Knowledge Graph for relational memory, and a shared Blackboard for multi-agent coordination.
- **RAG Pipeline** — Retrieval-Augmented Generation with OpenAI-compatible embeddings, configurable chunking strategies, and importance-based consolidation.
- **GDPR-Ready** — Data export, memory hygiene, and automatic cleanup.

### Security
- **Comprehensive Sandboxing** — Docker, Firejail, Bubblewrap, Landlock (Linux kernel), and Seatbelt (macOS) for process isolation.
- **Access Control** — Role-based access control (RBAC), policy-based IAm authorization, and Nevis enterprise IAM integration.
- **Secret Management** — Encrypted credential storage using ChaCha20-Poly1305, HMAC webhook verification.
- **Operational Safety** — Emergency stop switches (kill-all, network-kill, domain-block, tool-freeze) for immediate incident response.
- **Prompt Injection Defense** — Leak detector, taint tracking, and domain allowlisting.

### Hardware & IoT
- **Board Support** — STM32 Nucleo-F401RE, Raspberry Pi GPIO, Arduino Uno bridge via serial.
- **Peripheral Abstraction** — Trait-driven hardware API for adding new boards.
- **IoT Protocols** — MQTT broker integration for sensor/actuator control.

### Extensibility
- **Plugin System** — WASM-based plugin runtime via Extism for extending functionality.
- **A2A Protocol** — Standardized agent-to-agent communication for multi-vendor agent ecosystems.
- **MCP (Model Context Protocol)** — Built-in MCP tool bridge.
- **Composio Integration** — Direct access to the Composio tool registry.

### Observability
- **Prometheus Metrics** — Built-in metrics endpoint.
- **OpenTelemetry** — Distributed tracing support.
- **Health Endpoints** — `/health` and `/metrics` for monitoring.

---

## Architecture

SenAgentOS follows a trait-driven, modular architecture. Every major component — providers, channels, tools, memory backends, peripherals, and runtime adapters — is an implementation of a well-defined Rust trait. Adding support for a new provider, channel, or tool means implementing a trait and registering it in the factory module.

### Key Design Decisions

- **Rust-First** — 100% Rust across backend and embedded targets. Zero dependency on Python or Node.js for the agent core.
- **Trait-Driven** — Every extension point is a trait. Adding a new provider, channel, tool, or memory backend requires implementing a trait and registering in the factory.
- **Memory-First** — First-class memory management with multiple backends, vector search, importance scoring, and consolidation.
- **Security by Default** — Sandboxing, RBAC, secret storage, and prompt injection defense are built-in, not bolted on.
- **Autonomous Evolution** — Agents can create new skills, refine prompts, and improve their own behavior based on feedback.

---

## Building from Source

### Prerequisites

- **Rust 1.87+** (edition 2024)
- **Node.js 20+** (for the web frontend)
- **CMake** (for some native dependencies)

#### 1. Visual Studio Build Tools (Windows only)

On Windows, native dependencies require the MSVC toolchain and Windows SDK:

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

During installation (or via the Visual Studio Installer), select the **"Desktop development with C++"** workload.

#### 2. Rust Toolchain

**Linux / macOS:**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
```

**Windows:**

```powershell
winget install Rustlang.Rustup
# Open a new terminal after installation, then run:
rustup default stable
```

#### 3. Verify

```bash
rustc --version
cargo --version
```

### Clone the Repository

```bash
git clone https://github.com/senweaver/SenAgentOS.git
cd SenAgentOS
```

### Backend Only (No Frontend)

If you only need the CLI agent without the web dashboard:

```bash
cargo build --release
```

The binary will be at `target/release/senagentos`.

### Full Build (Backend + Frontend)

The build script automatically bundles the React frontend into the binary, so a single `senagentos` executable serves both the CLI and the web dashboard.

```bash
cargo build --release --features default
```

> **Note:** The first build compiles the Rust backend (this takes 5–20 minutes depending on your machine). Subsequent builds use incremental compilation and are much faster.

### Frontend Development Mode

For active frontend development, you can run the Vite dev server alongside the gateway for hot module replacement:

```bash
# Terminal 1: Start the gateway (serves the current API)
./target/release/senagentos gateway start

# Terminal 2: Run the frontend dev server (port 5173)
cd web
npm install
npm run dev
```

The frontend dev server proxies API requests to the gateway at `http://localhost:42617`.

### Production Frontend Build

Build only the frontend for deployment:

```bash
cd web
npm install
npm run build
```

Output is in `web/dist/`.

---

## Configuration

SenAgentOS uses a `config.toml` file. Copy the example and customize:

```bash
cp .env.example .env
# Edit .env or create ~/.senagent/config.toml
```

### Key Environment Variables

| Variable | Description | Default |
|---|---|---|
| `PROVIDER` | LLM provider name | `openrouter` |
| `SENWEAVER_API_KEY` | API key for SenWeaver | — |
| `SENWEAVER_BASE_URL` | Base URL for SenWeaver | `https://api3.senweaver.com/v1` |
| `OPENAI_API_KEY` | OpenAI API key | — |
| `ANTHROPIC_API_KEY` | Anthropic API key | — |
| `SENAGENTOS_WORKSPACE` | Working directory | `~/.senagent/workspace` |

### Quick Start with SenWeaver (Recommended)

SenWeaver provides a unified API that aggregates multiple providers:

```bash
# Set environment variables
export SENWEAVER_API_KEY="your-api-key"
export SENWEAVER_BASE_URL="https://api3.senweaver.com/v1"
export PROVIDER="senweaver"

# Interactive onboarding
./target/release/senagentos onboard --quick

# Or start the gateway with web dashboard
./target/release/senagentos gateway start
```

### Quick Start with OpenRouter

```bash
export OPENROUTER_API_KEY="your-api-key"
export PROVIDER="openrouter"
./target/release/senagentos agent
```

### Configuration File

For production deployments, use `~/.senagent/config.toml`:

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

Run `./target/release/senagentos config --help` for the full configuration reference.

---

## Running

### CLI Agent Mode

Run the agent interactively from the terminal:

```bash
./senagentos agent
```

Pass a message directly:

```bash
./senagentos agent --message "What's the weather in Tokyo?"
```

### Gateway Mode (Web Dashboard)

Start the HTTP/WebSocket gateway:

```bash
./senagentos gateway start
```

The gateway provides:
- **Web Dashboard** — Open `http://localhost:42617/` in your browser
- **REST API** — `http://localhost:42617/api/*`
- **WebSocket Chat** — `ws://localhost:42617/ws/chat`
- **Health Check** — `http://localhost:42617/health`
- **Prometheus Metrics** — `http://localhost:42617/metrics`

### Channels

Run individual channel bots:

```bash
./senagentos channel telegram --token "your-bot-token"
./senagentos channel discord --token "your-bot-token"
./senagentos channel slack --token "xoxb-..."
```

### Self-Test

Verify your setup:

```bash
./senagentos self-test
```

---

## Frontend Dashboard

The web dashboard provides a full GUI for managing the agent:

| Page | Description |
|---|---|
| **Agent Chat** | Interactive chat with streaming responses |
| **Dashboard** | System status, resource usage, active agents |
| **Tools** | Browse and manage available tools |
| **Cron** | Schedule and manage recurring tasks |
| **Memory** | Browse the agent's memory store |
| **Config** | Edit configuration inline |
| **Logs** | Real-time log viewer with filtering |
| **Doctor** | Health diagnostics and troubleshooting |
| **Integrations** | Manage channel connections |
| **Skills** | View and manage agent skills |
| **Hardware** | Monitor and control peripherals |
| **Canvas** | Shared visual workspace for agents |
| **Settings** | User preferences |

Tech stack: React 19, React Router 7, TypeScript, Tailwind CSS 4, Vite 6, Lucide React icons.

---

## Adding New Integrations

SenAgentOS is designed to be extended. The main extension points are:

- **Provider** — Implement a new LLM provider by adding a module in `src/providers/` and registering it in the factory.
- **Channel** — Add a new messaging platform by creating a module in `src/channels/` and implementing the `Channel` trait.
- **Tool** — Implement custom tools by creating a module in `src/tools/` and implementing the `Tool` trait.
- **Memory Backend** — Add a new memory backend by creating a module in `src/memory/` and implementing the `Memory` trait.

---

## Security

Review the [security documentation](docs/security/) before deploying in production. Key considerations:

- **Sandboxing** — Enable at least one sandbox backend (`sandbox-docker`, `sandbox-firejail`, etc.) to isolate tool execution.
- **RBAC** — Configure role-based access control for tools and channels.
- **Pairing** — Use device pairing for gateway access instead of open access.
- **Secrets** — Never commit API keys. Use the secret store or environment variables.
- **E-Stop** — Familiarize yourself with the emergency stop commands for incident response.

### Sandboxing Options

| Backend | Linux | macOS | Windows |
|---|---|---|---|
| Docker | ✓ | — | — |
| Firejail | ✓ | — | — |
| Bubblewrap | ✓ | — | — |
| Landlock | ✓ | — | — |
| Seatbelt | — | ✓ | — |

---

## Project Status

SenAgentOS is under active development. The core agent, gateway, and all major integrations are functional. Some features are still maturing:

- **Skill Evolution** — Autonomous skill creation is operational but the skill marketplace is a roadmap item.
- **Hardware** — Board support for STM32 and RPi is functional; Arduino bridge is stable.
- **Plugin System** — WASM plugin runtime is integrated but plugin SDK is in progress.
- **A2A Protocol** — Core implementation is complete; ecosystem adoption is in early stages.

---

## Contributing

Contributions are welcome. Please read the [contributing guidelines](docs/contributing/) before submitting PRs.

- Work from a non-`master` branch. Open PRs to `master`.
- Follow conventional commit titles (`feat:`, `fix:`, `docs:`, etc.).
- Run validation before opening PRs: `cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo test`.

---

## License

MIT License. See [LICENSE](LICENSE).
