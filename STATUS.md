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

### Frontend (React SPA)
- Project lijst met create/delete
- Project detail met tabs (Issues, Import/Export, API Keys)
- Topic lijst met status/priority badges, create/delete
- Topic detail met beschrijving, comments, viewpoints
- Comment thread met add/delete
- Viewpoint cards met snapshot preview en camera info
- BCF ZIP import (drag & drop) en export (download)
- API key management (create, copy, revoke)
- OIDC login flow (token opslaan, meesturen)
- Responsive layout met 3BM brand colors
- Vite dev proxy naar backend

### Backend (bcf-server)
- Volledige REST API voor projects, topics, comments, viewpoints
- BCF v2.1 conforme endpoints (`/bcf/2.1/...`)
- Platform endpoints (`/api/v1/...`)
- Multipart BCF ZIP import/export
- Snapshot storage (filesystem, base64 via API)
- OIDC authenticatie + session JWT + API keys
- Structured logging, CORS, error handling

### Infrastructuur
- Docker Compose (dev + prod)
- Multi-stage Dockerfile (frontend build + Rust build)
- Caddy: SPA routing + API reverse proxy
- Deploy script, auto-migrations

## Wat ontbreekt

- **Role-based access control**: schema aanwezig, enforcement nog niet
- **Event/audit log endpoints**: schema bestaat, endpoints niet
- **Project extensions endpoints**: custom enums per project
- **Input validatie**: alleen basis checks
- **Tests**: minimale coverage
- **Koppeling met OpenAEC Validator**
