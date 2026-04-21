-- Users: enforce UNIQUE(email).
--
-- `find_by_email` previously used `ORDER BY created_at ASC LIMIT 1` as a
-- workaround because email was not unique. That workaround introduces a
-- race condition during Authentik forward_auth auto-provisioning: two
-- concurrent requests for the same unknown email could both insert a row
-- and subsequent lookups would silently prefer the older one.
--
-- This migration adds `CONSTRAINT users_email_unique UNIQUE(email)` so the
-- database guarantees a single canonical row per email address.
--
-- Safety: if duplicate emails already exist we ABORT the migration with a
-- descriptive error. Duplicates must be resolved manually (merge rows,
-- reassign foreign keys on `projects.created_by`, `topics.assigned_to`,
-- `topics.creation_author`, `topics.modified_author`, `comments.author_id`,
-- `events.author_id`, `api_keys.created_by`, `project_members.user_id`)
-- before this constraint can be applied. Deleting rows automatically would
-- risk orphaning historical authorship records.

DO $$
DECLARE
  dup_count INTEGER;
  dup_sample TEXT;
BEGIN
  SELECT COUNT(*), COALESCE(string_agg(email, ', '), '')
    INTO dup_count, dup_sample
  FROM (
    SELECT email
    FROM users
    GROUP BY email
    HAVING COUNT(*) > 1
    LIMIT 10
  ) AS dups;

  IF dup_count > 0 THEN
    RAISE EXCEPTION
      'Migration 003 aborted: % duplicate email(s) in users table (sample: %). '
      'Resolve manually by merging rows and reassigning foreign keys, then rerun migrations.',
      dup_count, dup_sample;
  END IF;
END
$$;

ALTER TABLE users
  ADD CONSTRAINT users_email_unique UNIQUE (email);
