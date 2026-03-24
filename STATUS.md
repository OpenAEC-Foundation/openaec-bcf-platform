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

### Frontend (OpenAEC ribbon UI — volledig conform referentie)
- AppBar (40px): brand wordmark, quick-access (Save, Settings), user menu
- Ribbon (122px): animated tab indicator, slide-animaties bij tab switch
- Tabs: Bestand (amber, Backstage), Start (Project/Issues/Acties), Beeld
- Backstage: amber back button, Import/Export BCF, About panel met brand identity
- StatusBar (22px): project/topic count, connectie-indicator
- Keyboard shortcuts: Escape (sluit backstage), Ctrl+I (import), Ctrl+E (export)
- Brand systeem: brand.ts + injectBrandStyles.ts (runtime CSS var injection)
- Full themes.css: 150+ CSS vars, light + openaec dark theme
- Design System v0.4: Space Grotesk, Inter, JetBrains Mono
- Project CRUD, topic lijst/detail, comments, viewpoints, API keys
- BCF ZIP import (drag & drop) en export (download)
- OIDC login flow via Authentik

### Backend (bcf-server)
- Volledige REST API (BCF v2.1 + platform endpoints)
- OIDC auth + session JWT (24h) + API keys (bcfk_xxx)
- Lazy JWKS fetch, static file serving met SPA fallback
- Structured logging, CORS, error handling

### Auth (live)
- Authentik OIDC provider (bcf-platform app)
- AUTH_ENABLED=true, login werkt
- JIT user provisioning

### Infrastructuur
- Docker Compose met auth env vars
- Multi-stage Dockerfile (Node + Rust)
- Caddy reverse proxy
- Live op https://bcf.open-aec.com

## Wat ontbreekt

- Role-based access control (schema aanwezig, enforcement niet)
- Event/audit log endpoints
- Project extensions endpoints
- Input validatie (BCF-specifiek)
- Tests (minimale coverage)
- Koppeling met OpenAEC Validator
- i18n (labels hardcoded NL)
