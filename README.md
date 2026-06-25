# 📖 Bookmark Manager

一个基于 Rust 生态构建的高性能、轻量级书签管理 Web 应用。

> [在线体验](https://bks.artista.cc)
> * **用户名**：`guest`
> * **密码**：`guest`

## 🚀 特性

* **极致性能**：基于 Rust 的 `axum` 框架与 PostgreSQL 构建，内存占用低、响应速度快。
* **现代化全栈体验**：结合 `htmx` 与 `askama` 模板引擎，无需引入重量级前端框架即可实现无刷新的动态交互。
* **优雅 UI**：采用 TailwindCSS 构建，界面现代、布局响应式。
* **生产级部署**：内置开箱即用的 Nginx、Systemd 配置，并支持多环境配置分离。

## 🛠️ 技术栈

* **后端**：Rust（`axum`）
* **数据库**：PostgreSQL + Redis（用于缓存与会话管理）
* **模板引擎**：Askama（编译期类型安全的 HTML 模板）
* **前端**：htmx + TailwindCSS + Bun（构建期工具）

---

## 💻 本地开发

如需在本地进行开发或二次开发，请确保已安装 **Rust**、**Bun** 和 **PostgreSQL**。

### 1. 初始化数据库

在 PostgreSQL 中创建名为 `bookmark` 的数据库，并执行迁移脚本：

```bash
psql -U postgres -d postgres -c "CREATE DATABASE bookmark;"
# 执行数据库迁移
psql -U postgres -d bookmark < migrations/20260624051018_init.sql
```

### 2. 编译前端资源

项目使用 `bun` 与 TailwindCSS 管理前端依赖并编译静态资源：

```bash
bun install
bun run dev
```

### 3. 启动后端服务

```bash
cargo run
```

> **🔑 默认管理员账号**
> 服务启动后，访问 `http://127.0.0.1:8000`，使用以下默认凭证登录：
> * **用户名**：`admin`
> * **密码**：`admin`（建议登录后立即修改）

---

## 📦 生产部署

生产环境部署十分简单，只需从 Release 页面下载对应的压缩包即可。

### 1. 解压发布包

从 Release 页面下载 `bookmark.zip`，并解压到服务器目标目录（如 `/app/bookmark`）：

```bash
unzip bookmark.zip -d /app/bookmark
cd /app/bookmark
```

解压后的目录结构如下：

```text
.
├── bookmark                  # 编译好的二进制可执行文件
├── ddl.sql                   # 数据库初始化脚本
├── configuration/
│   └── base.yaml              # 配置文件
├── public/                    # 静态资源目录（JS / CSS / 图片）
├── nginx.conf                 # Nginx 示例配置
└── bookmark.service           # Systemd 服务配置文件
```

### 2. 修改配置

编辑 `configuration/base.yaml`，根据服务器实际情况调整路径与凭证：

```yaml
application:
  port: 8000
  host: 0.0.0.0
  static_directory: "/app/bookmark/public"   # 修改为实际的绝对路径
  upload_directory: "/app/bookmark/upload"   # 修改为实际的上传路径
  image_quality: 80.0                        # 图片转换为 WebP 时的压缩质量，取值范围 0.0-100.0
database:
  host: "127.0.0.1"
  port: 5432
  username: "postgres"
  password: "your_secure_password"
  database_name: "bookmark"
  require_ssl: false
redis_uri: "redis://127.0.0.1:6379"
```

### 3. 初始化生产数据库

确保生产环境的 PostgreSQL 已创建 `bookmark` 数据库，并执行迁移脚本：

```bash
psql -U postgres -d bookmark < ddl.sql
```

### 4. 使用 Systemd 管理服务

将自带的 `bookmark.service` 移动到系统的 systemd 目录并启动：

```bash
# 1. 复制服务文件
sudo cp bookmark.service /etc/systemd/system/

# 2. 重新加载 systemd 配置
sudo systemctl daemon-reload

# 3. 启动服务并设置开机自启
sudo systemctl enable --now bookmark

# 4. 查看运行状态
sudo systemctl status bookmark
```

> **💡 提示**：请确保 `bookmark.service` 中的 `ExecStart` 与 `WorkingDirectory` 指向实际的解压路径（如 `/app/bookmark`）。

### 5. Nginx 反向代理

如需启用自定义域名或 SSL，可参考附带的 `nginx.conf` 进行配置，并将其合并到你的 Nginx 配置中（通常位于 `/etc/nginx/sites-available/`）：

```nginx
server {
    listen 80;
    server_name yourdomain.com;

    location / {
        proxy_pass http://127.0.0.1:8000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

配置完成后，重启 Nginx：

```bash
sudo nginx -s reload
```

---

## 🔒 初始登录信息

服务启动成功后，请使用系统预设的管理员账号首次登录：

| 字段 | 初始值 |
| --- | --- |
| **用户名（Username）** | `admin` |
| **密码（Password）** | `admin` |

> **⚠️ 安全提示**：为保障数据安全，请在首次登录成功后立即前往设置页面修改默认密码。

---

## 📄 开源协议

本项目采用 [MIT](LICENSE) 协议开源。

## 联系方式

如有疑问，请联系开发者。

![联系作者](images/ccwechat.jpg)
