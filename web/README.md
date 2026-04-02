# SenAgentOS Web Dashboard

智能体操作系统的 Web 管理界面。

## 快速开始

### 1. 启动后端服务

在项目根目录启动 SenAgentOS 网关服务：

```bash
# 设置环境变量
$env:SENWEAVER_API_KEY = "your_senweaver_api_key"
$env:SENWEAVER_BASE_URL = "https://api3.senweaver.com/v1"
$env:PROVIDER = "senweaver"

# 启动网关服务 (端口 42617)
./target/release/senagentos.exe gateway start
```

### 2. 启动 Web 开发服务器

```bash
cd web

# 安装依赖
npm install

# 启动开发服务器 (端口 5173)
npm run dev
```

### 3. 访问应用

打开浏览器访问: http://localhost:5173

## 生产构建

```bash
# 构建生产版本
npm run build

# 前端将输出到 web/dist 目录
```

## 功能模块

- **Agent Chat** - 与智能体对话
- **Dashboard** - 系统状态仪表盘
- **Tools** - 工具管理
- **Cron** - 定时任务管理
- **Memory** - 记忆存储
- **Config** - 配置管理
- **Logs** - 日志查看
- **Doctor** - 系统诊断

## 技术栈

- React 19
- React Router 7
- TypeScript
- Tailwind CSS 4
- Vite 6
- Lucide React (图标)
- React Markdown (Markdown 渲染)
