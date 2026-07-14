# ref-server — agent guide

## Dev environment

- **Nix + devenv** required. Enter shell: `devenv shell` (or `.envrc` handles it via direnv).
- PostgreSQL runs via `just up` (start: `just up`; stop: Ctrl+C).
- Never commit `.env` secrets. `.env` is loaded automatically by devenv.

## Key commands (via `just`)

| Command | Notes |
|---|---|
| `just check` | `treefmt` then `cargo check` |
| `just fmt` | `treefmt` — multiplexes `rustfmt` + `nixfmt` via `.treefmt.toml` |
| `just lint` | `cargo clippy -- -D warnings` |
| `just test-all` | Spins up testing environment and runs all tests |
| `just test-unit` | Runs unit tests. Quick, no background services needed |
| `just up` | Start background services (PostgreSQL, static-web-server, Prometheus) — required before integration tests |
| `just run` | `cargo run` — API server on `API_LISTEN_SOCKET` (default `0.0.0.0:3001`) |
| `just ci` | `check` → `lint` → `test-all` |
| `just ci-quick` | `check` → `lint` → `test-unit` |
| `just db-setup` | `diesel setup` (creates DB + runs all migrations) |
| `just db-migration-new <name>` | Scaffold a new migration via diesel CLI |
| `just db-migration-run` / `db-migration-redo` | Run / redo migrations |
| `just db-clean-state` | Nuke `.devenv/state/postgres` (e.g. after pg version bump) |

## Architecture

- Single crate (`ref-server`), no workspace.
- Binary entrypoint: `src/bin/server.rs` (default-run = `"server"`).
- Library root: `src/lib.rs` — re-exports modules, defines `DbPool`.
- Routes: `src/routes/mod.rs::router(pool)` builds the axum `Router`.
- All errors flow through `app::Error` in `src/app.rs` — produces `{"error": "message"}` JSON.
- Migrations are embedded and run at startup only with `--database-run-migrations` flag.

## Testing

- **Always use `cargo nextest`** (never `cargo test`). Each test runs in its own process, so global-state issues (e.g. axum-prometheus recorder) never arise.
- **Unit tests** live inside source files (`#[cfg(test)] mod tests`). Run with `cargo nextest run --lib`.
- **Integration tests** in `tests/` use `common::test_server()` which builds a real DB-backed router with `begin_test_transaction()` isolation — no cleanup needed.
- Test framework: `googletest` (`assert_that!`, `eq`, `some`, `contains_substring`, etc.).

## Database

- **`src/schema.rs` is auto-generated** by `diesel print-schema`. After creating or modifying a migration, regenerate with `diesel print-schema`.
- DB connection via Unix socket (`$DATABASE_URL` set by devenv). Migrations use sync `PgConnection` in `spawn_blocking` because libpq parses Unix-socket URLs differently from tokio-postgres.

## Conventions

- Rust edition 2024.
- Format before commit: `just fmt` (pre-commit hook enforces via treefmt).
- clippy must pass with `-D warnings` (pre-commit hook enforces).
- No codegen steps. No build script.
- Commit messages: concise, matching repo style.
- Docs in Org-mode with `verb-mode` annotations (in `docs/`).
