-- Project enhancements: location, image, local users

-- Add location and image to projects
ALTER TABLE projects ADD COLUMN location TEXT NOT NULL DEFAULT '';
ALTER TABLE projects ADD COLUMN image_path TEXT;

-- Allow local platform users (no OIDC sub required)
ALTER TABLE users ALTER COLUMN sub DROP NOT NULL;
ALTER TABLE users ADD COLUMN password_hash TEXT;

-- Drop the unique constraint on sub (allow NULL duplicates)
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_sub_key;
CREATE UNIQUE INDEX idx_users_sub ON users (sub) WHERE sub IS NOT NULL;

-- Ensure user has either OIDC sub or local password
ALTER TABLE users ADD CONSTRAINT users_auth_method
  CHECK (sub IS NOT NULL OR password_hash IS NOT NULL);
