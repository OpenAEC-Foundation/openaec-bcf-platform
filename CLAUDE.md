# CLAUDE.md - OpenAEC BCF Platform

## Project: BCF Issue Management Platform

### Quick Summary
Standalone BCF 2.1 issue management platform (a la BIMcollab Online). Centraal issue management voor BIM-projecten, met import/export van .bcfzip en koppeling naar de OpenAEC Validator.

### Tech Stack
- **Backend:** Rust (Axum 0.8, Tokio, sqlx)
- **Database:** PostgreSQL 16
- **Auth:** Authentik OIDC (Fase 4)
- **BCF parsing:** quick-xml + zip crate (bcf-core library)
- **Frontend:** React 18, TypeScript, Vite, TailwindCSS (Fase 5)

### Workspace Structure
```
crates/
  bcf-core/    — BCF 2.1 types, XML/ZIP parsing (no web deps)
  bcf-server/  — Axum web server, API routes, database layer
frontend/      — React SPA (Fase 5)
migrations/    — PostgreSQL schema migrations
```

### Commands
```bash
# Build
cargo build --workspace

# Test
cargo test --workspace

# Clippy (must be warning-free)
cargo clippy --workspace

# Run server (requires PostgreSQL)
docker compose up -d
cargo run -p bcf-server

# Run migrations only
sqlx migrate run --source migrations
```

### API Routes
- BCF v2.1: `/bcf/2.1/projects`, `/bcf/2.1/projects/{id}/topics`, etc.
- Platform: `/api/v1/projects`, `/api/v1/projects/{id}/import-bcf`, etc.
- Health: `/health`

### 3BM Brand Colors
- Magic Violet: #350E35
- Verdigris: #44B6A8
- Friendly Yellow: #EFBD75
- Warm Magenta: #A01C48
- Flaming Peach: #DB4C40

### Code Style
- `cargo clippy` must pass with zero warnings
- All public items get `///` doc comments
- Modules get `//!` header docs
- Parameter structs over long argument lists
- Constants for magic values

---

## Agent Broker
- **project_id:** `bcf-platform`
- **display_name:** `BCF Platform`
- **capabilities:** `["bcf-issues", "rust", "axum"]`
- **subscriptions:** `["bim/*", "shared/*"]`

---

## Orchestrator

Bij sessie START → lees `X:\10_3BM_bouwkunde\50_Claude-Code-Projects\lessons_learned_global.md`
Bij sessie EINDE → schrijf `C:\Users\JochemK\.claude\orchestrator\sessions\bcf-platform_latest.md`
**Registry:** `C:\Users\JochemK\.claude\orchestrator\project-registry.json`
