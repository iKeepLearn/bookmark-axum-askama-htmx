# =============================================================
# Stage 1: 构建（Rust + bun 在同一阶段，因为 build.rs 在
#           `cargo build` 期间会调用 bun/tailwindcss 生成前端资源，
#           两者是耦合的，不能拆成互相独立的两个 stage）
# =============================================================
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    curl \
    unzip \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 安装 bun（build.rs 编译期会调用它）
RUN curl -fsSL https://bun.sh/install | bash
ENV PATH="/root/.bun/bin:${PATH}"

# 先拷贝依赖清单，利用 Docker 层缓存
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

# sqlx 离线模式：编译期不需要连接真实数据库，用仓库里预生成的
# .sqlx 查询元数据文件即可（和 CI 里的 SQLX_OFFLINE=true 保持一致）
ENV SQLX_OFFLINE=true
ENV CARGO_TERM_COLOR=always

# 拷贝其余源码（包含 build.rs、askama 模板、.sqlx 缓存等）
COPY . .

RUN cargo build --release

# =============================================================
# Stage 2: 运行时镜像（尽量精简，不含 Rust/bun 工具链）
# =============================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    postgresql-client \
    gettext-base \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 编译好的二进制
COPY --from=builder /app/target/release/bookmark /app/bookmark

# 前端产物 + 静态资源目录（对齐 build.yml 打包脚本里的目录清单）
COPY --from=builder /app/public   /app/public
COPY --from=builder /app/static   /app/static
COPY --from=builder /app/locales  /app/locales

# 数据库迁移脚本（幂等 SQL，容器启动时自动执行）
COPY migrations /app/migrations

# 配置模板（用环境变量渲染成最终的 base.yaml）
COPY docker/base.yaml.template /app/configuration/base.yaml.template

# 启动脚本
COPY docker/entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

RUN mkdir -p /app/upload

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8000/ || exit 1

ENTRYPOINT ["/app/entrypoint.sh"]
