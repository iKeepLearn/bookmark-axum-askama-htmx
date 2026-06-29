# 📖 Bookmark Manager

A high-performance, lightweight bookmark manager built on the Rust ecosystem.

> [online site](https://bks.artista.cc)
> * **用户名**：`guest`
> * **密码**：`guest`

## 📸 Screenshots

### Web Features

| Feature | Screenshot |
| --- | --- |
| 🏠 Home | ![Web Home](images/web-home.png) |
| ➕ Add Bookmark | ![Add Bookmark](images/web-add.png) |
| ✏️ Edit Bookmark | ![Edit Bookmark](images/web-edit.png) |
| 📥 Import Bookmarks | ![Import Bookmarks](images/web-import.png) |

### Browser Extension

| Feature | Screenshot |
| --- | --- |
| ➕ Add Bookmark | ![Extension Add Bookmark](images/ext-add.png) |
| ⚙️ Extension Settings | ![Extension Settings](images/ext-setting.png) |

### Install and Use the Browser Extension

#### Using the Compiled Version (Recommended)

A pre-compiled extension package `bookmark-clipper-1.1.0-chrome.zip` is provided in the project root directory. Follow these steps to install:

1. Download `bookmark-clipper-1.1.0-chrome.zip` to any directory
2. Open Chrome/Edge browser and visit `chrome://extensions` (Edge: `edge://extensions`)
3. Enable "Developer mode" in the top right corner
4. Drag and drop the downloaded extension zip file into the Chrome extensions page
5. After installation, click the extension icon in the browser toolbar to open the settings page

#### Configure the Extension

1. Click the settings icon in the top-right corner of the extension popup to open the settings page
2. In the settings page, enter your bookmark library URL (e.g., `https://bks.artista.cc`), username, and password, then click "Generate Key"
3. Once configured, you can save bookmarks in two ways:
   - **Click the extension icon**: Open the popup, edit the title, select category, add tags, then save
   - **Right-click menu**: Right-click on a page or link → "Save to Bookmark Library" for quick saving

#### Develop the Extension from Source

To modify the extension functionality, enter the `browser-ext` directory for development:

```bash
cd browser-ext
bun install
bun run dev      # Chrome dev mode with hot reload
```

After running `bun run dev`, follow the terminal prompt to load the `.output/chrome-mv3-dev` directory in `chrome://extensions`.

Build the extension:
```bash
bun run build    # Compiled output in .output/chrome-mv3
bun run zip      # Package into .zip file
```

## 🚀 Features

* **Blazing performance**: Built with the Rust `axum` framework and PostgreSQL — low memory footprint, fast response times.
* **Modern full-stack experience**: Combines `htmx` with the `askama` template engine to deliver dynamic, no-refresh interactions without a heavyweight frontend framework.
* **Polished UI**: Built with TailwindCSS for a modern, responsive interface.
* **Browser extension support**: Companion browser extension for one-click bookmark saving of current pages.
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
# Application configuration
application:
  # HTTP service listening port
  port: 8000
  # HTTP service listening address
  host: 0.0.0.0
  # Absolute path to static assets directory (JS/CSS/images, etc.)
  static_directory: "/app/bookmark/public"   # Set to the actual absolute path
  # Absolute path to file upload directory
  upload_directory: "/app/bookmark/upload"   # Set to the actual upload path
  # WebP compression quality for image conversion, range 0.0-100.0 (higher = better quality, larger files)
  image_quality: 80.0
# Database configuration
database:
  # PostgreSQL host address
  host: "127.0.0.1"
  # PostgreSQL port
  port: 5432
  # PostgreSQL username
  username: "postgres"
  # PostgreSQL password
  password: "your_secure_password"
  # PostgreSQL database name
  database_name: "bookmark"
  # Whether SSL connection is required
  require_ssl: false
# Redis connection URI (for caching and session management)
redis_uri: "redis://127.0.0.1:6379"
# API Token configuration (for external apps like browser extension)
api_token:
  # Secret key for signing tokens (use a randomly generated key in production)
  secret_key: "very-long-and-random-secret-key"
  # Token expiration time (supported units: s/seconds, m/minutes, h/hours, d/days)
  expire_time: 1h
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
