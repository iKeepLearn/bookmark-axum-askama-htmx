# 🐳 Docker 部署指南

最简单的自托管方式，无需手动安装 Rust / PostgreSQL / Redis，只需要 Docker 和 Docker Compose。

## 快速开始

有两种方式，推荐大多数用户用方式 A（更快，不用等本地编译 Rust）：

### 方式 A：使用预构建镜像（推荐）

```bash
# 1. 只需要 compose 文件和环境变量文件，不需要克隆整个仓库
mkdir bookmark-manager && cd bookmark-manager
curl -O https://raw.githubusercontent.com/ikeeplearn/bookmark-axum/main/docker-compose.ghcr.yml
curl -O https://raw.githubusercontent.com/ikeeplearn/bookmark-axum/main/.env.example

# 2. 复制环境变量文件并修改
cp .env.example .env
nano .env   # 至少修改 POSTGRES_PASSWORD 和 API_TOKEN_SECRET

# 3. 启动（直接从 GHCR 拉取镜像，几十秒搞定）
docker compose -f docker-compose.ghcr.yml up -d
```

### 方式 B：本地编译镜像

```bash
# 1. 克隆仓库
git clone https://github.com/ikeeplearn/bookmark-axum.git
cd bookmark-axum

# 2. 复制环境变量文件并修改
cp .env.example .env
nano .env   # 至少修改 POSTGRES_PASSWORD 和 API_TOKEN_SECRET

# 3. 构建并启动（Rust 编译较慢，第一次可能要几分钟）
docker compose up -d --build

# 4. 查看日志，确认启动成功
docker compose logs -f app
```

启动完成后访问 `http://your-server-ip:8000`，使用默认账号登录：

| 用户名 | 密码 |
| --- | --- |
| `admin` | `admin` |

**⚠️ 首次登录后请立即修改密码。**

## 目录结构说明

```text
.
├── docker-compose.yml
├── Dockerfile
├── .env                    # 你的配置（不要提交到 git）
├── .env.example
└── data/
    └── uploads/            # 用户上传的图片等文件，宿主机目录，方便直接备份
```

数据库和 Redis 数据存放在 Docker 具名卷中（`postgres-data`、`redis-data`），由 Docker 统一管理，不需要你手动操心目录权限问题。

## 常用操作

**备份数据库：**
```bash
docker compose exec postgres pg_dump -U bookmark bookmark > backup_$(date +%F).sql
```

**恢复数据库：**
```bash
cat backup_2026-07-01.sql | docker compose exec -T postgres psql -U bookmark -d bookmark
```

**更新到最新版本：**
```bash
git pull
docker compose build app
docker compose up -d
```

**查看应用日志：**
```bash
docker compose logs -f app
```

## 配合反向代理使用（推荐生产环境）

Docker Compose 只暴露了 `8000` 端口，建议在前面加一层反向代理来处理域名和 HTTPS。你可以用宿主机上现成的 Nginx（参考仓库自带的 `nginx.conf`），或者用 Caddy 自动申请证书：

```caddyfile
yourdomain.com {
    reverse_proxy localhost:8000
}
```

## 环境变量完整说明

| 变量名 | 说明 | 默认值 |
| --- | --- | --- |
| `APP_PORT` | 应用对外暴露的端口 | `8000` |
| `POSTGRES_USER` | 数据库用户名 | `bookmark` |
| `POSTGRES_PASSWORD` | 数据库密码（必填） | 无 |
| `POSTGRES_DB` | 数据库名 | `bookmark` |
| `API_TOKEN_SECRET` | 浏览器插件等外部访问的签名密钥（必填，建议用 `openssl rand -hex 32` 生成） | 无 |
| `API_TOKEN_EXPIRE` | Token 有效期 | `1h` |
| `IMAGE_QUALITY` | 封面图 WebP 压缩质量 | `80.0` |

## 常见问题

**Q: 容器起来了但访问 8000 端口没反应？**
先看日志 `docker compose logs -f app`，大概率是数据库连接失败或迁移报错，检查 `.env` 里的 `POSTGRES_PASSWORD` 是否和 postgres 服务实际使用的一致。

**Q: 想换端口怎么办？**
改 `.env` 里的 `APP_PORT`，然后 `docker compose up -d` 重启即可，不需要改 `docker-compose.yml`。

**Q: 上传的图片丢了？**
检查 `./data/uploads` 目录有没有被误删，这个目录是 bind mount，不在 Docker 卷里，删容器不会影响它，但删这个宿主机目录会导致数据丢失，注意纳入你的备份脚本。
