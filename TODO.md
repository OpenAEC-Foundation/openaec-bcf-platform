# TODO — OpenAEC BCF Platform

> Laatste update: 2026-03-24

## Fase 5 — Frontend

- [ ] React 18 + TypeScript + Vite project opzetten
- [ ] TailwindCSS met 3BM brand colors configureren
- [ ] Login flow (OIDC redirect via `/auth/login`)
- [ ] Auth context (JWT opslaan, meesturen in requests)
- [ ] Project lijst/detail views
- [ ] Topic lijst/detail/create/edit views
- [ ] Comment thread UI
- [ ] Viewpoint weergave (3D camera positie, snapshot preview)
- [ ] BCF ZIP upload (import) UI
- [ ] BCF ZIP download (export) UI
- [ ] API key management UI

## Backlog

- [ ] Role-based access control afdwingen op routes (middleware per role)
- [ ] Event/audit log endpoints toevoegen
- [ ] Project extensions endpoints (custom topic types, statuses, priorities, labels, stages)
- [ ] Input validatie uitbreiden (BCF-specifieke constraints)
- [ ] Test coverage verbeteren (integration tests, edge cases)
- [ ] Conflict resolution bij concurrent edits
- [ ] Koppeling met OpenAEC Validator
- [ ] OIDC token refresh flow
