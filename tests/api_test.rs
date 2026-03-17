//! Integration tests for BCF platform API.
//!
//! These tests require a running PostgreSQL instance.
//! Set DATABASE_URL environment variable before running.
//!
//! ```bash
//! docker compose up -d postgres
//! DATABASE_URL=postgres://bcf:bcf_dev_password@localhost:5432/bcf_platform cargo test
//! ```

// Integration tests will be run as separate binaries.
// For Fase 1, we rely on the unit tests in bcf-core and
// manual testing with curl against a running server.
//
// Full integration tests with test database will be added
// once the basic CRUD is verified working.
