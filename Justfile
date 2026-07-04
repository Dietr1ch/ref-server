# justfile for demo-server
# see https://just.systems/man/en/

export DATABASE_URL := env_var_or_default("DATABASE_URL", "postgres://demo:demo@localhost:5432/demo")
export LISTEN_SOCKET	:= env_var_or_default("LISTEN_SOCKET", "0.0.0.0:3000")



# Meta
# ====

# Show available recipes
default:
	@just --list



# Database
# ========

# Create the database and run migrations
setup: db-create migrate

# Create the database (idempotent)
db-create:
	diesel setup

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
# The data gets recreated on next `devenv up`.
db-clean-state:
	rm -rf .devenv/state/postgres



# Development
# ===========

# Start the development server
run:
	cargo run

# Run `cargo check` (fast compilation check)
check:
	cargo check

build:
	cargo build

# Run all tests
test:
	cargo test

# Run clippy lints
lint:
	cargo clippy -- -D warnings

# Format Rust and Nix code
fmt:
	cargo fmt
	nixpkgs-fmt *.nix

# Format code, then check it still compiles
fmt-check: fmt check

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

# Run the full CI pipeline: check, test, lint
ci: check test lint
