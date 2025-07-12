# Gemini API 密钥池

[English](README.md)

## 项目概述

本项目是一个使用 Rust 编写的轻量级、高性能代理服务器。它提供了一个与 OpenAI API 兼容的接口 (`/v1/chat/completions`)，并将接收到的请求智能地路由到 Google Gemini API。项目内置了一个 Gemini API 密钥池，每次请求都会轮换使用密钥，从而实现负载均衡和 API 配额的有效管理。

整个应用程序已通过 Docker 容器化，可实现一键轻松部署。

备注：本项目除了这行文字，其他均由Claude Code输出，可以任意使用 :)

## 功能特性

- **OpenAI API 兼容**: 可作为使用 OpenAI 聊天补全（Chat Completions）API 格式的服务的直接替代品。
- **API 密钥管理**: 动态创建、编辑和删除 API 密钥，并提供使用情况追踪。
- **Web 管理界面**: 现代化科技风格的 Web 仪表板，用于管理 API 密钥和监控使用情况。
- **使用分析**: 实时追踪请求数、输入/输出 token 数量和 API 密钥统计信息。
- **安全认证**: 基于 JWT 的管理员认证和基于角色的访问控制。
- **API 密钥轮换**: 每次请求都会从预定义的密钥池中自动轮换使用 Gemini API 密钥。
- **动态模型选择**: 根据请求体中的 `model` 字段，自动选择并调用不同的 Gemini 模型 (例如 `gemini-1.5-flash`, `gemini-1.5-pro`)。
- **高性能**: 基于 Rust 和 Axum 构建，提供异步、快速且可靠的性能。
- **轻松部署**: 使用 Docker 和一个简单的 Shell 脚本即可实现一键部署。
- **数据库存储**: 使用 SQLite 数据库持久化存储 API 密钥和使用日志。

## 环境要求

- [Docker](https://www.docker.com/get-started)
- [Git](https://git-scm.com/downloads)

## 快速开始

### 1. 克隆仓库

```bash
git clone https://github.com/cod3vil/gemini-pool.git
cd gemini-pool
```

### 2. 配置

应用程序需要一个 `.env` 文件来存储您的 Gemini API 密钥。

1.  **创建 `.env` 文件**，通过复制我们提供的示例文件：
    ```bash
    cp gemini-pool/.env.example gemini-pool/.env
    ```

2.  **编辑 `gemini-pool/.env`** 文件并配置必要的设置：

    ```dotenv
    # gemini-pool/.env

    # 在此处放置您的 Gemini API 密钥，用逗号分隔。
    GEMINI_API_KEYS=your_key_1,your_key_2,your_key_3

    # Web 管理界面的管理员认证
    ADMIN_USERNAME=admin
    ADMIN_PASSWORD=your_secure_admin_password

    # 管理员会话管理的 JWT 密钥（生产环境中请更改此值）
    JWT_SECRET=your_jwt_secret_change_in_production

    # 数据库 URL（默认使用 SQLite）
    DATABASE_URL=sqlite:./gemini_pool.db

    # 服务器在容器内部绑定的地址。
    # 为了能通过 Docker 从主机访问，此项必须是 0.0.0.0:8080。
    LISTEN_ADDR=0.0.0.0:8080
    ```

### 3. 使用 Docker 部署 (推荐)

我们提供的 `deploy.sh` 脚本可以处理从构建 Docker 镜像到运行容器的所有事务。

1.  **为脚本授予执行权限**: 
    ```bash
    chmod +x deploy.sh
    ```

2.  **运行脚本**: 
    - 使用默认端口 `8080` 运行：
      ```bash
      ./deploy.sh
      ```
    - 使用自定义端口 (例如 `9000`) 运行：
      ```bash
      ./deploy.sh 9000
      ```

该脚本将自动构建镜像，停止任何旧的同名容器，并在后台启动一个新容器。

- **查看日志**: `docker logs -f gemini-pool-container`
- **停止服务**: `docker stop gemini-pool-container`

## Web 管理界面

部署完成后，您可以访问 Web 管理界面来管理 API 密钥和监控使用情况：

### 访问控制面板

1. **登录页面**: 导航到 `http://127.0.0.1:8080/login.html`
2. **使用您的管理员凭据**（在您的 `.env` 文件中配置）
3. **访问管理控制面板** `http://127.0.0.1:8080/management.html`

### 功能特性

- **🎨 现代科技风格界面**: 赛博朋克风格设计，带有矩阵雨背景效果
- **📊 实时仪表板**: 查看 API 密钥总数、请求数、token 数量和活跃密钥数
- **🔑 API 密钥管理**: 
  - 创建新的 API 密钥（自动生成或自定义）
  - 编辑密钥名称和切换激活状态
  - 删除未使用的密钥
  - 查看每个密钥的详细使用统计
- **📈 使用分析**: 追踪输入/输出 token 和请求计数
- **🔒 安全认证**: 基于 JWT 的会话管理
- **📱 响应式设计**: 支持桌面和移动设备

### 客户端 API 密钥认证

使用 API 接口（`/v1/chat/completions`，`/v1/models`）时，客户端必须包含通过 Web 界面创建的 API 密钥：

```bash
curl -X POST http://127.0.0.1:8080/v1/chat/completions \
-H "Content-Type: application/json" \
-H "Authorization: Bearer your_client_api_key" \
-d '{...}'
```

## API 使用方法

向 `/v1/chat/completions` 接口发送一个 `POST` 请求。请求体应遵循 OpenAI Chat Completions API 的格式。

### 请求示例

这是一个使用 `curl` 与服务交互的示例。您可以在 `model` 字段中指定任何受支持的 Gemini 模型。

```bash
curl -X POST http://127.0.0.1:8080/v1/chat/completions \
-H "Content-Type: application/json" \
-H "Authorization: Bearer your_client_api_key" \
-d '{
  "model": "gemini-1.5-flash",
  "messages": [
    {
      "role": "system",
      "content": "你是一个乐于助人的助手。"
    },
    {
      "role": "user",
      "content": "你好！法国的首都是哪里？"
    }
  ]
}'
```

**注意**: 请将 `your_client_api_key` 替换为通过 Web 管理界面创建的 API 密钥。

服务器会使用您的一个密钥将此请求转发到 Gemini API，并以 OpenAI 的格式返回响应。

### 列出模型

要查看此代理支持的可用模型列表，请向 `/v1/models` 接口发送一个 `GET` 请求。

```bash
curl -H "Authorization: Bearer your_client_api_key" \
     http://127.0.0.1:8080/v1/models
```
