#!/usr/bin/env bash
# OpenAEC BCF Platform — Server deployment script
# Run on a fresh Ubuntu 24.04 server (Hetzner AX102)
#
# Usage: ssh root@46.224.215.142 'bash -s' < deploy.sh

set -euo pipefail

APP_DIR="/opt/openaec-bcf-platform"
REPO_URL="https://github.com/OpenAEC-Foundation/openaec-bcf-platform.git"

echo "=== OpenAEC BCF Platform — Deployment ==="

# --- 1. System updates ---
echo "[1/6] Updating system packages..."
apt-get update -qq
apt-get upgrade -y -qq

# --- 2. Install Docker (if not present) ---
if ! command -v docker &>/dev/null; then
  echo "[2/6] Installing Docker..."
  curl -fsSL https://get.docker.com | sh
  systemctl enable docker
  systemctl start docker
else
  echo "[2/6] Docker already installed: $(docker --version)"
fi

# Ensure Docker Compose plugin is available
if ! docker compose version &>/dev/null; then
  echo "ERROR: Docker Compose plugin not found"
  exit 1
fi

# --- 3. Clone or update repository ---
echo "[3/6] Setting up application..."
if [ -d "$APP_DIR" ]; then
  echo "  Repository exists, pulling latest..."
  cd "$APP_DIR"
  git pull --ff-only
else
  echo "  Cloning repository..."
  git clone "$REPO_URL" "$APP_DIR"
  cd "$APP_DIR"
fi

# --- 4. Generate production .env if not exists ---
echo "[4/6] Configuring environment..."
if [ ! -f "$APP_DIR/.env" ]; then
  # Generate a secure random password for PostgreSQL
  PG_PASSWORD=$(openssl rand -base64 24 | tr -dc 'a-zA-Z0-9' | head -c 32)

  cat > "$APP_DIR/.env" <<EOL
POSTGRES_PASSWORD=${PG_PASSWORD}
EOL

  echo "  Generated .env with secure PostgreSQL password"
  echo "  IMPORTANT: Back up this password!"
  echo "  PostgreSQL password: ${PG_PASSWORD}"
else
  echo "  .env already exists, keeping existing configuration"
fi

# --- 5. Build and start services ---
echo "[5/6] Building and starting services..."
cd "$APP_DIR"
docker compose -f docker-compose.prod.yml build --no-cache
docker compose -f docker-compose.prod.yml up -d

# Wait for services to be healthy
echo "  Waiting for services to start..."
sleep 10

# --- 6. Verify deployment ---
echo "[6/6] Verifying deployment..."

# Check if containers are running
if docker compose -f docker-compose.prod.yml ps --format json | grep -q '"running"'; then
  echo "  Containers are running"
else
  echo "  WARNING: Some containers may not be running"
  docker compose -f docker-compose.prod.yml ps
fi

# Test health endpoint (internal)
HEALTH=$(docker compose -f docker-compose.prod.yml exec -T bcf-server curl -sf http://localhost:3000/health 2>/dev/null || echo "FAIL")
if echo "$HEALTH" | grep -q '"ok"'; then
  echo "  Health check: OK"
else
  echo "  Health check: PENDING (server may still be starting)"
  echo "  Check logs: docker compose -f docker-compose.prod.yml logs bcf-server"
fi

echo ""
echo "=== Deployment complete ==="
echo ""
echo "Services:"
docker compose -f docker-compose.prod.yml ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}"
echo ""
echo "Next steps:"
echo "  1. Point DNS: bcf.open-aec.com -> 46.224.215.142"
echo "  2. Caddy will auto-provision SSL once DNS propagates"
echo "  3. Test: curl https://bcf.open-aec.com/health"
echo "  4. Logs: cd $APP_DIR && docker compose -f docker-compose.prod.yml logs -f"
