# Gemini API 密钥池

## 项目概述

本项目是一个使用 Rust 编写的轻量级、高性能代理服务器。它提供了一个与 OpenAI API 兼容的接口 (`/v1/chat/completions`)，并将接收到的请求智能地路由到 Google Gemini API。项目内置了一个 Gemini API 密钥池，每次请求都会轮换使用密钥，从而实现负载均衡和 API 配额的有效管理。

整个应用程序已通过 Docker 容器化，可实现一键轻松部署。

备注：本项目除了这行文字，其他均由Claude Code输出，可以任意使用 :)

## 功能特性

- **OpenAI API 兼容**: 可作为使用 OpenAI 聊天补全（Chat Completions）API 格式的服务的直接替代品。
- **API 密钥轮换**: 每次请求都会从预定义的密钥池中自动轮换使用 Gemini API 密钥。
- **动态模型选择**: 根据请求体中的 `model` 字段，自动选择并调用不同的 Gemini 模型 (例如 `gemini-1.5-flash`, `gemini-1.5-pro`)。
- **高性能**: 基于 Rust 和 Axum 构建，提供异步、快速且可靠的性能。
- **轻松部署**: 使用 Docker 和一个简单的 Shell 脚本即可实现一键部署。
- **可配置端口**: 可以轻松更改服务运行时所映射的主机端口。

## 环境要求

- [Docker](https://www.docker.com/get-started)
- [Git](https://git-scm.com/downloads)

## 快速开始

### 1. 克隆仓库

```bash
git clone <repository-url>
cd gemini-pool
```

### 2. 配置

应用程序需要一个 `.env` 文件来存储您的 Gemini API 密钥。

1.  **创建 `.env` 文件**，通过复制我们提供的示例文件：
    ```bash
    cp gemini-pool/.env.example gemini-pool/.env
    ```

2.  **编辑 `gemini-pool/.env`** 文件并添加您的密钥。密钥应使用逗号分隔。

    ```dotenv
    # gemini-pool/.env

    # 在此处放置您的 Gemini API 密钥，用逗号分隔。
    GEMINI_API_KEYS=your_key_1,your_key_2,your_key_3

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

## API 使用方法

向 `/v1/chat/completions` 接口发送一个 `POST` 请求。请求体应遵循 OpenAI Chat Completions API 的格式。

### 请求示例

这是一个使用 `curl` 与服务交互的示例。您可以在 `model` 字段中指定任何受支持的 Gemini 模型。

```bash
curl -X POST http://127.0.0.1:8080/v1/chat/completions \
-H "Content-Type: application/json" \
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

服务器会使用您的一个密钥将此请求转发到 Gemini API，并以 OpenAI 的格式返回响应。

### 列出模型

要查看此代理支持的可用模型列表，请向 `/v1/models` 接口发送一个 `GET` 请求。

```bash
curl http://127.0.0.1:8080/v1/models
```
