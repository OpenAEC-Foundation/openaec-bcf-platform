#!/usr/bin/env bash
# OpenAEC BCF Platform — Deployment script
# Run from the server: cd /opt/openaec/openaec-bcf-platform && ./deploy.sh
#
# Prerequisites:
# - Docker + Docker Compose plugin
# - openaec_default network exists
# - Global Caddy routes bcf.open-aec.com → bcf-server:3000

set -euo pipefail

APP_DIR="/opt/openaec/openaec-bcf-platform"

echo "=== OpenAEC BCF Platform — Deploy ==="

# --- 1. Pull latest code ---
echo "[1/4] Pulling latest code..."
cd "$APP_DIR"
git pull --ff-only

# --- 2. Generate .env if not exists ---
echo "[2/4] Checking environment..."
if [ ! -f "$APP_DIR/.env" ]; then
  PG_PASSWORD=$(openssl rand -base64 24 | tr -dc 'a-zA-Z0-9' | head -c 32)
  cat > "$APP_DIR/.env" <<EOL
POSTGRES_PASSWORD=${PG_PASSWORD}
EOL
  echo "  Generated .env — PostgreSQL password: ${PG_PASSWORD}"
  echo "  IMPORTANT: Back up this password!"
else
  echo "  .env exists, keeping current configuration"
fi

# --- 3. Build and restart ---
echo "[3/4] Building and starting services..."
docker compose -f docker-compose.prod.yml build --no-cache
docker compose -f docker-compose.prod.yml up -d

echo "  Waiting for services to start..."
sleep 10

# --- 4. Verify ---
echo "[4/4] Verifying deployment..."

docker compose -f docker-compose.prod.yml ps --format "table {{.Name}}\t{{.Status}}"

HEALTH=$(docker compose -f docker-compose.prod.yml exec -T bcf-server curl -sf http://localhost:3000/health 2>/dev/null || echo "FAIL")
if echo "$HEALTH" | grep -q '"ok"'; then
  echo "  Health check: OK"
else
  echo "  Health check: PENDING (server may still be starting)"
  echo "  Logs: docker compose -f docker-compose.prod.yml logs bcf-server"
fi

echo ""
echo "=== Deploy complete ==="
echo "  URL: https://bcf.open-aec.com"
echo "  Logs: docker compose -f docker-compose.prod.yml logs -f bcf-server"
