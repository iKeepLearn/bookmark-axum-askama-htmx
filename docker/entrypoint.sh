#!/usr/bin/env bash
set -euo pipefail

# =============================================================
# 1. 等待 Postgres 就绪
# =============================================================
echo "[entrypoint] waiting for postgres at ${POSTGRES_HOST}:${POSTGRES_PORT} ..."
until pg_isready -h "${POSTGRES_HOST}" -p "${POSTGRES_PORT}" -U "${POSTGRES_USER}" > /dev/null 2>&1; do
  sleep 1
done
echo "[entrypoint] postgres is ready."

# =============================================================
# 2. 用环境变量渲染最终的 base.yaml
#    （容器每次启动都重新渲染，方便用户改 .env 后 docker compose restart 即可生效）
#
#    注意：Docker Compose 读取 .env 文件时不会像 shell 那样自动去掉引号，
#    如果用户在 .env 里写了 API_TOKEN_SECRET="xxx"，引号会被当成值的一部分，
#    渲染进 YAML 后会变成 "" xxx "" 导致嵌套解析错误。这里做一层防御性清理，
#    不管用户 .env 里加没加引号都能正常工作。
# =============================================================
strip_quotes() {
  local val="$1"
  val="${val%\"}"
  val="${val#\"}"
  val="${val%\'}"
  val="${val#\'}"
  echo "$val"
}

export POSTGRES_HOST POSTGRES_PORT REDIS_HOST REDIS_PORT
export POSTGRES_USER="$(strip_quotes "${POSTGRES_USER}")"
export POSTGRES_PASSWORD="$(strip_quotes "${POSTGRES_PASSWORD}")"
export POSTGRES_DB="$(strip_quotes "${POSTGRES_DB}")"
export POSTGRES_REQUIRE_SSL="$(strip_quotes "${POSTGRES_REQUIRE_SSL}")"
export API_TOKEN_SECRET="$(strip_quotes "${API_TOKEN_SECRET}")"
export API_TOKEN_EXPIRE="$(strip_quotes "${API_TOKEN_EXPIRE}")"
export IMAGE_QUALITY="$(strip_quotes "${IMAGE_QUALITY}")"

export PGPASSWORD="${POSTGRES_PASSWORD}"

envsubst < /app/configuration/base.yaml.template > /app/configuration/base.yaml
echo "[entrypoint] configuration rendered."

# =============================================================
# 3. 自动执行数据库迁移
#    migrations 目录下的 SQL 全部使用 IF NOT EXISTS / ADD COLUMN IF NOT EXISTS 写法，
#    可以安全地重复执行，因此每次启动都跑一遍，新用户首次启动会自动建表。
# =============================================================
echo "[entrypoint] applying migrations ..."
for f in /app/migrations/*.sql; do
  echo "[entrypoint]   -> $(basename "$f")"
  psql -h "${POSTGRES_HOST}" -p "${POSTGRES_PORT}" -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" \
       -v ON_ERROR_STOP=1 -f "$f"
done
echo "[entrypoint] migrations applied."

# =============================================================
# 4. 启动服务
# =============================================================
echo "[entrypoint] starting bookmark server ..."
exec /app/bookmark
