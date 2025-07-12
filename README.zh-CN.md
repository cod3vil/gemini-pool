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

1. **主页**: 导航到 `http://127.0.0.1:8080/` - 双语（中文/English）功能概览页面，提供管理面板直接链接
2. **管理界面**: 访问 `http://127.0.0.1:8080/admin` - 管理界面主页
3. **登录页面**: `http://127.0.0.1:8080/admin/login.html`
4. **管理控制面板**: `http://127.0.0.1:8080/admin/management.html`
5. **使用您的管理员凭据**（在您的 `.env` 文件中配置）

### API 接口结构

所有管理 API 接口现在统一组织在 `/admin/api/` 路径下：

- **身份验证**: 
  - `POST /admin/api/auth/login` - 管理员登录
  - `GET /admin/api/auth/verify` - Token 验证
- **控制面板**: `GET /admin/api/dashboard` - 统计信息
- **API 密钥管理**: 
  - `GET /admin/api/api-keys` - 列出 API 密钥
  - `POST /admin/api/api-keys` - 创建 API 密钥
  - `GET /admin/api/api-keys/{id}` - 获取特定 API 密钥
  - `PUT /admin/api/api-keys/{id}` - 更新 API 密钥
  - `DELETE /admin/api/api-keys/{id}` - 删除 API 密钥

### 功能特性

- **🎨 现代科技风格界面**: 赛博朋克风格设计，带有矩阵雨背景效果
- **🌍 双语支持**: 完整的中英文界面，支持持久化语言偏好设置
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

## 使用 Nginx 进行生产部署

对于生产环境，建议在 Gemini Pool 服务前使用 nginx 作为反向代理。这可以提供额外的安全性、SSL 终止和负载均衡功能。

### 简化的 Nginx 配置

得益于重新组织的路由结构（`/admin/api/*` 路径）和专用首页，您可以使用非常简单的 nginx 配置：

```nginx
server {
    listen 80;
    server_name your-domain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;
    
    # SSL 配置
    ssl_certificate /path/to/your/fullchain.pem;
    ssl_certificate_key /path/to/your/private.key;
    
    # 安全头设置
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    
    # 将所有请求代理到应用程序
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # 用于长时间运行的请求
        proxy_read_timeout 60s;
        client_max_body_size 10M;
    }
}
```

### 高级 Nginx 配置 (带速率限制)

创建一个 nginx 配置文件 (`/etc/nginx/sites-available/gemini-pool`)：

```nginx
server {
    listen 80;
    server_name your-domain.com;
    
    # 将 HTTP 重定向到 HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;
    
    # SSL 配置
    ssl_certificate /path/to/your/fullchain.pem;
    ssl_certificate_key /path/to/your/private.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;
    
    # 安全头设置
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    
    # API 接口 (速率限制)
    location /v1/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # 增加长时间运行请求的超时时间
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
        
        # 处理大型请求体
        client_max_body_size 10M;
    }
    
    # 管理 API 接口 (更严格的速率限制)
    location /api/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
    
    # Web 管理界面
    location /admin/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
    
    # 将根路径重定向到管理员登录页面
    location = / {
        return 301 /admin/login.html;
    }
    
    # 静态文件和其他路径
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
    
    # 健康检查接口
    location /health {
        access_log off;
        proxy_pass http://127.0.0.1:8080/;
        proxy_set_header Host $host;
    }
    
    # 日志记录
    access_log /var/log/nginx/gemini-pool.access.log;
    error_log /var/log/nginx/gemini-pool.error.log;
}
```

### 设置 Nginx

1. **安装 Nginx** (如果尚未安装)：
   ```bash
   # Ubuntu/Debian
   sudo apt update && sudo apt install nginx
   
   # CentOS/RHEL
   sudo yum install nginx
   ```

2. **配置速率限制区域** (选择一种方法)：

   **方法 A: 添加到主 nginx.conf**
   ```bash
   sudo nano /etc/nginx/nginx.conf
   ```
   在 `http` 块内添加 `limit_req_zone` 指令：
   ```nginx
   http {
       # ... 现有配置 ...
       
       # Gemini Pool 的速率限制区域
       limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
       limit_req_zone $binary_remote_addr zone=admin:10m rate=5r/s;
       
       # ... 其余配置 ...
   }
   ```

   **方法 B: 创建单独的区域文件**
   ```bash
   sudo nano /etc/nginx/conf.d/gemini-pool-zones.conf
   ```
   添加速率限制区域：
   ```nginx
   limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
   limit_req_zone $binary_remote_addr zone=admin:10m rate=5r/s;
   ```

3. **创建服务器配置文件**：
   ```bash
   sudo nano /etc/nginx/sites-available/gemini-pool
   ```
   添加服务器配置 (如果使用上述方法 A 或 B，则不包含 `limit_req_zone` 行)。

4. **启用站点**：
   ```bash
   sudo ln -s /etc/nginx/sites-available/gemini-pool /etc/nginx/sites-enabled/
   ```

5. **测试配置**：
   ```bash
   sudo nginx -t
   ```

6. **重新加载 Nginx**：
   ```bash
   sudo systemctl reload nginx
   ```

### SSL 证书设置

对于生产使用，使用 Let's Encrypt 获取 SSL 证书：

```bash
# 安装 Certbot
sudo apt install certbot python3-certbot-nginx

# 获取证书
sudo certbot --nginx -d your-domain.com

# 自动续期 (添加到 crontab)
0 12 * * * /usr/bin/certbot renew --quiet
```

### 使用 Docker Compose 与 Nginx

对于完整的生产环境设置，您可以使用 Docker Compose：

```yaml
# docker-compose.yml
version: '3.8'

services:
  gemini-pool:
    build: ./gemini-pool
    container_name: gemini-pool-app
    env_file: ./gemini-pool/.env
    volumes:
      - gemini-pool-data:/data
    networks:
      - internal
    restart: unless-stopped
    
  nginx:
    image: nginx:alpine
    container_name: gemini-pool-nginx
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/conf.d/default.conf
      - ./ssl:/etc/nginx/ssl
      - /var/log/nginx:/var/log/nginx
    networks:
      - internal
    depends_on:
      - gemini-pool
    restart: unless-stopped

volumes:
  gemini-pool-data:

networks:
  internal:
    driver: bridge
```
