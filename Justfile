# justfile for ref-server
# see https://just.systems/man/en/


# Meta
# ====

# Show available recipes
default: check
	@just --list



# Database
# ========

# Run pending migrations
migrate:
	diesel migration run

# Redo the last migration (rollback then re-apply)
redo:
	diesel migration redo

# Create a new migration with a given name, e.g. `just migration add_posts`
migration name:
	diesel migration generate {{name}}

# Drop and recreate the database, run all migrations
db-reset:
	diesel database reset
	diesel migration run

# Nuke devenv's PostgreSQL state (e.g. after switching pg versions).
# The data gets recreated on next `devenv processes up`.
db-clean-state:
	rm -rf .devenv/state/postgres



# Development
# ===========

# Start the development server
run:
	cargo run

# Run `cargo check` (fast compilation check)
check: fmt
	cargo check

build:
	cargo build

# Run all tests (requires `devenv processes up` for integration tests)
test:
	cargo nextest run

# Run fast unit tests (no database needed)
test-unit:
	cargo nextest run --lib

# Run clippy lints
lint:
	cargo clippy -- -D warnings

# Format Rust and Nix code
fmt:
	cargo fmt
	nixfmt --strict *.nix

# Open Rust docs for dependencies
doc:
	cargo doc --open

update:
	nix flake update
	cargo update


# Clean build artifacts
clean:
	cargo clean



# CI
# ==

# Run the full CI pipeline (requires PostgreSQL via `devenv processes up`)
ci: check test lint

# Quick CI — unit tests only, no database needed
ci-quick: check test-unit lint
