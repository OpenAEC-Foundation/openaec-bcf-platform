# STATUS — OpenAEC BCF Platform

> Laatste update: 2026-03-24

## Overzicht Fases

| Fase | Omschrijving | Status |
|------|-------------|--------|
| **1** | Foundation (Rust/Axum, PostgreSQL, basis API) | ✅ Done |
| **2** | Viewpoint CRUD + snapshot storage | ✅ Done |
| **3** | BCF ZIP import/export | ✅ Done |
| **4** | Authenticatie (Authentik OIDC + API keys) | ✅ Done |
| **5** | Frontend (React 18, TypeScript, Vite, Tailwind) | ✅ Done |

## Wat werkt

### Frontend (React SPA — OpenAEC ribbon UI)
- AppBar (40px): donker chrome, "OpenAEC" brand wordmark, user menu (login/logout)
- Ribbon toolbar (122px): Bestand tab (amber, opent Backstage), Start tab, Beeld tab
- Backstage overlay: BCF Import/Export, Over, Sluiten
- StatusBar (22px): project/topic count, connectie-indicator
- OpenAEC Foundation design system (DESIGN-SYSTEM.md v0.4)
- Fonts: Space Grotesk (headings), Inter (body), JetBrains Mono (code)
- Kleuren: amber accenten, deep-forge chrome, blueprint-white content
- Project lijst met create/delete
- Project detail met page tabs (Issues, Import/Export, API Keys)
- Topic lijst met status/priority badges, create/delete
- Topic detail met beschrijving, comments, viewpoints
- Comment thread met add/delete
- Viewpoint cards met snapshot preview en camera info
- BCF ZIP import (drag & drop) en export (download)
- API key management (create, copy, revoke)
- OIDC login flow (token opslaan, meesturen)

### Backend (bcf-server)
- Volledige REST API voor projects, topics, comments, viewpoints
- BCF v2.1 conforme endpoints (`/bcf/2.1/...`)
- Platform endpoints (`/api/v1/...`)
- Multipart BCF ZIP import/export
- Snapshot storage (filesystem, base64 via API)
- OIDC authenticatie + session JWT (24h) + API keys (bcfk_xxx)
- Lazy JWKS fetch (geen crash bij lege JWKS)
- Static file serving met SPA fallback (/app/static → index.html)
- Structured logging, CORS, error handling

### Auth (live op productie)
- Authentik OIDC provider geconfigureerd (bcf-platform app)
- AUTH_ENABLED=true op server
- Login via https://auth.open-aec.com
- JIT user provisioning vanuit OIDC claims
- API key systeem voor service-to-service auth

### Infrastructuur
- Docker Compose (dev + prod) met auth env vars
- Multi-stage Dockerfile (Node frontend + Rust backend)
- Caddy reverse proxy (direct naar bcf-server, geen Authentik forward auth)
- Deploy script, auto-migrations bij startup
- Live op https://bcf.open-aec.com

## Wat ontbreekt

- **Role-based access control**: schema aanwezig, enforcement nog niet
- **Event/audit log endpoints**: schema bestaat, endpoints niet
- **Project extensions endpoints**: custom enums per project
- **Input validatie**: alleen basis checks
- **Tests**: minimale coverage
- **Koppeling met OpenAEC Validator**
- **i18n**: labels nu hardcoded NL, geen taalwisseling
