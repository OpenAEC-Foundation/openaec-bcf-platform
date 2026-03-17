-- OpenAEC BCF Platform — Initial Schema
-- Alle tabellen gebruiken UUID primary keys en timestamps.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Users (JIT provisioned via OIDC)
CREATE TABLE users (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  sub        TEXT UNIQUE NOT NULL,          -- OIDC subject claim
  email      TEXT NOT NULL,
  name       TEXT NOT NULL,
  avatar_url TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_users_email ON users (email);

-- Projects
CREATE TABLE projects (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name        TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  created_by  UUID REFERENCES users (id),
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Project membership + roles
CREATE TABLE project_members (
  project_id UUID NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
  user_id    UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  role       TEXT NOT NULL DEFAULT 'member'
             CHECK (role IN ('owner', 'admin', 'member', 'viewer')),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (project_id, user_id)
);

-- Project extensions (custom enums per project)
CREATE TABLE project_extensions (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id   UUID NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
  topic_types  JSONB NOT NULL DEFAULT '[]',
  topic_statuses JSONB NOT NULL DEFAULT '["Open", "Closed"]',
  priorities   JSONB NOT NULL DEFAULT '["Normal"]',
  labels       JSONB NOT NULL DEFAULT '[]',
  stages       JSONB NOT NULL DEFAULT '[]',
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_extensions_project ON project_extensions (project_id);

-- Topics (BCF issues)
CREATE TABLE topics (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id      UUID NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
  title           TEXT NOT NULL,
  description     TEXT NOT NULL DEFAULT '',
  topic_type      TEXT NOT NULL DEFAULT '',
  topic_status    TEXT NOT NULL DEFAULT 'Open',
  priority        TEXT NOT NULL DEFAULT 'Normal',
  assigned_to     UUID REFERENCES users (id),
  stage           TEXT NOT NULL DEFAULT '',
  labels          JSONB NOT NULL DEFAULT '[]',
  due_date        DATE,
  creation_author UUID REFERENCES users (id),
  modified_author UUID REFERENCES users (id),
  index_number    INTEGER,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_topics_project ON topics (project_id);
CREATE INDEX idx_topics_status ON topics (project_id, topic_status);
CREATE INDEX idx_topics_assigned ON topics (assigned_to);

-- Comments
CREATE TABLE comments (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  topic_id   UUID NOT NULL REFERENCES topics (id) ON DELETE CASCADE,
  author_id  UUID REFERENCES users (id),
  comment    TEXT NOT NULL,
  viewpoint_id UUID,  -- FK added after viewpoints table
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_comments_topic ON comments (topic_id);

-- Viewpoints (camera + snapshot + component visibility)
CREATE TABLE viewpoints (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  topic_id        UUID NOT NULL REFERENCES topics (id) ON DELETE CASCADE,
  snapshot_path   TEXT,
  camera          JSONB,    -- { type, position, direction, up, fov, aspect }
  components      JSONB,    -- { visibility, selection, coloring }
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_viewpoints_topic ON viewpoints (topic_id);

-- Add FK from comments to viewpoints
ALTER TABLE comments
  ADD CONSTRAINT fk_comments_viewpoint
  FOREIGN KEY (viewpoint_id) REFERENCES viewpoints (id) ON DELETE SET NULL;

-- Events (audit log)
CREATE TABLE events (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  topic_id   UUID NOT NULL REFERENCES topics (id) ON DELETE CASCADE,
  author_id  UUID REFERENCES users (id),
  event_type TEXT NOT NULL,    -- 'status_changed', 'assigned', 'comment_added', etc.
  old_value  TEXT,
  new_value  TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_events_topic ON events (topic_id);
CREATE INDEX idx_events_created ON events (created_at);

-- API keys (service-to-service auth, e.g. validator → platform)
CREATE TABLE api_keys (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id  UUID NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
  name        TEXT NOT NULL,
  key_hash    TEXT NOT NULL,    -- bcrypt hash of the key
  prefix      TEXT NOT NULL,    -- first 8 chars for identification (bcfk_xxx)
  created_by  UUID REFERENCES users (id),
  expires_at  TIMESTAMPTZ,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_api_keys_prefix ON api_keys (prefix);
CREATE INDEX idx_api_keys_project ON api_keys (project_id);
