# justfile for ref-server
# see https://just.systems/man/en/


# Meta
# ====

# Show available recipes
default: check
	@just --list



# Database
# ========

# Initialise diesel
db-setup:
	diesel setup

# Create a new migration with a given name, e.g. `just migration add_posts`
db-migration-new name:
	diesel migration generate {{name}}

# Run pending migrations
db-migration-run:
	diesel migration run

# Redo the last migration (rollback then re-apply to verify down+up)
db-migration-redo:
	diesel migration redo

# Drop and recreate the database, run all migrations
db-reset:
	diesel database reset
	diesel migration run

# Nuke devenv's PostgreSQL state (e.g. after switching pg versions).
# The data gets recreated on next `just up`.
db-clean-state:
	rm -rf .devenv/state/postgres


# Services
# ========

# Start up background services (PostgreSQL, static-web-server)
up:
	devenv processes up



# Development
# ===========

# Start the development server
run:
	cargo run

# Run `cargo check` (fast compilation check)
check: fmt
	cargo check

# Run `cargo build`
build:
	cargo build

# Run all tests (requires `just up` for integration tests)
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
	treefmt

# Open Rust docs for dependencies
doc:
	cargo doc --open

# Update nix flakes and cargo crates
update:
	nix flake update
	cargo update


# Clean build artifacts
clean:
	cargo clean



# CI
# ==

# Run the full CI pipeline (requires PostgreSQL via `just up`)
ci: check lint test

# Quick CI — unit tests only, no database needed
ci-quick: check lint test-unit
