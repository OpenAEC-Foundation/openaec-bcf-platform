# STATUS -- OpenAEC BCF Platform

> Laatste update: 2026-03-30

## Overzicht Fases

| Fase | Omschrijving | Status |
|------|-------------|--------|
| **1** | Foundation (Rust/Axum, PostgreSQL, basis API) | Done |
| **2** | Viewpoint CRUD + snapshot storage | Done |
| **3** | BCF ZIP import/export | Done |
| **4** | Authenticatie (Authentik OIDC + API keys) | Done |
| **5** | Frontend (React 18, TypeScript, Vite, Tailwind) | Done |
| **6** | Cloud migratie naar openaec-cloud crate | Done |

## Wat werkt

### Cloud Storage (openaec-cloud crate integratie)
- Gedeelde `openaec-cloud` crate als git dependency
- Multi-tenant support via `tenants.json` (TENANTS_CONFIG env var)
- Backward compatibility: legacy single-tenant env vars (NEXTCLOUD_URL etc.)
- Hybrid I/O: volume mount reads (snel) + WebDAV writes (NC in sync)
- Nieuw pad: `Projects/{project}/issues/` (was: `99_overige_documenten/bcf-platform/`)
- Automatische fallback: leest eerst nieuw pad, dan legacy pad
- Schrijft altijd naar nieuw pad (`issues/`)
- Models endpoint: `GET /api/cloud/projects/{project}/models` (IFC bestanden uit `models/`)
- Manifest endpoint: `GET /api/cloud/projects/{project}/manifest` (project.wefc)
- Bij BCF save naar cloud: automatische manifest update met WefcIssueSet object
- Frontend CloudPanel bijgewerkt voor nieuwe structuur

### Frontend (OpenAEC ribbon UI -- volledig conform referentie)
- **OpenAEC Design System:** Donker thema (#36363E), amber accent (#D97706), lichte tekst (#FAFAF9)
- Shared package: `@openaec/ui` (tokens.css + components.css) geinstalleerd
- Alle pagina-componenten gestyled met donkere achtergrond en amber-getinte borders
- StatusBadge, KPI cards, forms, inputs, selects: volledig dark theme
- Scrollbars met amber tint, select color-scheme: dark
- AppBar (40px): brand wordmark, quick-access (Save, Settings), user menu
- Ribbon (122px): animated tab indicator, slide-animaties bij tab switch
- Tabs: Bestand (amber, Backstage), Start (Project/Issues/Acties), Beeld
- Backstage: amber back button, Import/Export BCF, About panel met brand identity
- StatusBar (22px): project/topic count, connectie-indicator
- Keyboard shortcuts: Escape (sluit backstage), Ctrl+I (import), Ctrl+E (export)
- Brand systeem: brand.ts + injectBrandStyles.ts (runtime CSS var injection)
- Full themes.css: 150+ CSS vars, dark theme als default
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
- Docker Compose met auth + cloud env vars
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
