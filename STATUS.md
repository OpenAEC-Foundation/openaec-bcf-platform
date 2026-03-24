# STATUS — OpenAEC BCF Platform

> Laatste update: 2026-03-24

## Overzicht Fases

| Fase | Omschrijving | Status |
|------|-------------|--------|
| **1** | Foundation (Rust/Axum, PostgreSQL, basis API) | ✅ Done |
| **2** | Viewpoint CRUD + snapshot storage | ✅ Done |
| **3** | BCF ZIP import/export | ✅ Done |
| **4** | Authenticatie (Authentik OIDC + API keys) | ✅ Done |
| **5** | Frontend (React 18, TypeScript, Vite, Tailwind) | ❌ Niet gestart |

## Wat werkt

### Backend (bcf-server)
- Volledige REST API voor projects, topics, comments, viewpoints
- BCF v2.1 conforme endpoints (`/bcf/2.1/...`)
- Platform endpoints (`/api/v1/...`)
- Multipart BCF ZIP import (genereert nieuwe UUIDs)
- BCF ZIP export
- Snapshot storage (filesystem, base64 via API)
- Health check endpoint
- Structured logging (tracing)
- Error handling met automatische HTTP status mapping
- CORS enabled

### Authenticatie (Fase 4)
- **OIDC login flow**: `/auth/login` → Authentik redirect → `/auth/callback` → JWT sessie
- **Session JWT**: 24-uur tokens, `Authorization: Bearer <jwt>`
- **API keys**: `bcfk_xxx` format, bcrypt hashing, prefix-based lookup
- **AuthUser extractor**: Automatisch JWT of API key validatie in handlers
- **JIT user provisioning**: Gebruikers worden automatisch aangemaakt vanuit OIDC claims
- **Backwards compatible**: `AUTH_ENABLED=false` (default) = open access mode
- **API key management**: CRUD endpoints (`/api/v1/projects/{id}/api-keys`)
- **PKCE**: Authorization code flow met S256 PKCE
- Handlers vullen `created_by`, `creation_author`, `author_id` bij authenticatie

### Core library (bcf-core)
- BCF 2.1 types (Project, Topic, Comment, Viewpoint, Camera, Components)
- XML parsing (markup.bcf, viewpoint.bcfv)
- ZIP lezen/schrijven

### Database
- PostgreSQL 16 schema met 8 tabellen
- Migratie: `001_initial_schema.sql`
- Roles: owner/admin/member/viewer (schema aanwezig, enforcement in handlers)

### Infrastructuur
- Docker Compose (dev + prod)
- Multi-stage Dockerfile met dependency caching
- Caddy reverse proxy config
- Deploy script
- Migraties draaien automatisch bij startup

## Wat ontbreekt

- **Frontend**: geen `frontend/` map aanwezig
- **Role-based access control**: schema aanwezig, enforcement op route-niveau nog niet
- **Event/audit log endpoints**: schema bestaat, endpoints niet
- **Project extensions endpoints**: custom enums per project (schema aanwezig)
- **Input validatie**: alleen basis checks, geen BCF-specifieke constraints
- **Tests**: één testbestand (`tests/api_test.rs`), minimale coverage
