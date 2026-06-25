# 📖 Bookmark Manager

A high-performance, lightweight bookmark manager built on the Rust ecosystem.

> [online site](https://bks.artista.cc)
> * **用户名**：`guest`
> * **密码**：`guest`

## 🚀 Features

* **Blazing performance**: Built with the Rust `axum` framework and PostgreSQL — low memory footprint, fast response times.
* **Modern full-stack experience**: Combines `htmx` with the `askama` template engine to deliver dynamic, no-refresh interactions without a heavyweight frontend framework.
* **Polished UI**: Built with TailwindCSS for a modern, responsive interface.
* **Production-ready deployment**: Ships with ready-to-use Nginx and Systemd configs, with support for separate environment configurations.

## 🛠️ Tech Stack

* **Backend**: Rust (`axum`)
* **Database**: PostgreSQL + Redis (for caching and session management)
* **Templating**: Askama (compile-time, type-safe HTML templates)
* **Frontend**: htmx + TailwindCSS + Bun (build-time tooling)

---

## 💻 Local Development

To develop or contribute locally, make sure you have **Rust**, **Bun**, and **PostgreSQL** installed.

### 1. Initialize the database

Create a database named `bookmark` in PostgreSQL and run the migration:

```bash
psql -U postgres -d postgres -c "CREATE DATABASE bookmark;"
# Run the database migration
psql -U postgres -d bookmark < migrations/20260624051018_init.sql
```

### 2. Build frontend assets

The project uses `bun` and TailwindCSS to manage frontend dependencies and build static assets:

```bash
bun install
bun run dev
```

### 3. Run the backend

```bash
cargo run
```

> **🔑 Default admin account**
> Once the server is running, open `http://127.0.0.1:8000` and log in with the default credentials:
> * **Username**: `admin`
> * **Password**: `admin` (please change this immediately after logging in)

---

## 📦 Production Deployment

Deploying to production is straightforward — just download the release archive from the Releases page.

### 1. Extract the release package

Download `bookmark.zip` from the Releases page and extract it to your target directory on the server (e.g. `/app/bookmark`):

```bash
unzip bookmark.zip -d /app/bookmark
cd /app/bookmark
```

The extracted directory structure looks like this:

```text
.
├── bookmark                   # Compiled binary executable
├── ddl.sql                    # Database initialization scripts
├── configuration/
│   └── base.yaml              # Configuration file
├── public/                    # Static assets directory (JS / CSS / images)
├── nginx.conf                 # Sample Nginx configuration
└── bookmark.service           # Systemd service configuration
```

### 2. Update the configuration

Edit `configuration/base.yaml` to match your server's actual paths and credentials:

```yaml
application:
  port: 8000
  host: 0.0.0.0
  static_directory: "/app/bookmark/public"   # Set to the actual absolute path
  upload_directory: "/app/bookmark/upload"   # Set to the actual upload path
  image_quality: 80.0                        # WebP compression quality for image conversion, range 0.0-100.0
database:
  host: "127.0.0.1"
  port: 5432
  username: "postgres"
  password: "your_secure_password"
  database_name: "bookmark"
  require_ssl: false
redis_uri: "redis://127.0.0.1:6379"
```

### 3. Initialize the production database

Make sure the `bookmark` database exists on your production PostgreSQL instance, then run the migration:

```bash
psql -U postgres -d bookmark < ddl.sql
```

### 4. Manage the service with Systemd

Move the included `bookmark.service` file into the systemd directory and start the service:

```bash
# 1. Copy the service file
sudo cp bookmark.service /etc/systemd/system/

# 2. Reload systemd configuration
sudo systemctl daemon-reload

# 3. Start the service and enable it on boot
sudo systemctl enable --now bookmark

# 4. Check the service status
sudo systemctl status bookmark
```

> **💡 Tip**: Make sure `ExecStart` and `WorkingDirectory` in `bookmark.service` point to your actual extraction path (e.g. `/app/bookmark`).

### 5. Nginx reverse proxy

To enable a custom domain or SSL, you can use the included `nginx.conf` as a reference and merge it into your Nginx configuration (typically located at `/etc/nginx/sites-available/`):

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

After updating the configuration, reload Nginx:

```bash
sudo nginx -s reload
```

---

## 🔒 Initial Login Credentials

Once the service starts successfully, log in for the first time using the default admin account:

| Field | Default Value |
| --- | --- |
| **Username** | `admin` |
| **Password** | `admin` |

> **⚠️ Security notice**: For the safety of your data, please change the default password immediately after your first successful login.

---

## 📄 License

This project is open-sourced under the [MIT](LICENSE) license.
