use std::time::Duration;

use clap::Parser;
use diesel::Connection;
use diesel::pg::PgConnection;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_migrations::{MigrationHarness, embed_migrations};
use eyre::Context;
use tracing_subscriber::EnvFilter;

use ref_server::routes;

/// Embedded Diesel migrations (run automatically on startup).
const MIGRATIONS: diesel_migrations::EmbeddedMigrations = embed_migrations!("migrations");

/// Configuration read from environment variables.
#[derive(Parser)]
#[command(
	name = "ref-server",
	version,
	about = "axum + diesel + PostgreSQL reference implementation"
)]
struct Config {
	// Database
	/// Postgres connection string
	#[arg(env = "DATABASE_URL")]
	database_url: String,
	/// Whether to run migrations during startup
	#[arg(long)]
	database_run_migrations: bool,

	/// The connection timeout
	#[arg(
    long,
    default_value = "5s",
    value_parser = humantime::parse_duration
	)]
	database_connect_timeout: Duration,
	/// The pool size
	#[arg(long, default_value = "8")]
	database_pool_size: u32,

	// Server
	/// Socket address (host:port) to listen on.
	#[arg(env = "API_LISTEN_SOCKET", default_value = "0.0.0.0:3001")]
	listen_socket: String,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let config = Config::parse();

	// Structured logging
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_env("SERVER_LOG"))
		.init();

	// Database
	// --------
	let connection_config =
		AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.database_url);
	let pool = Pool::builder()
		.max_size(config.database_pool_size)
		.connection_timeout(config.database_connect_timeout)
		.build(connection_config)
		.await
		.wrap_err("Failed to build connection pool")?;

	// Run pending migrations at startup.
	// Uses sync Diesel/libpq (spawn_blocking) rather than tokio-postgres because
	// tokio-postgres parses the connection URL differently from libpq, which can
	// cause failures with Unix-socket-based connection strings.
	if config.database_run_migrations {
		let database_url = config.database_url.clone();
		tokio::task::spawn_blocking(move || {
			let mut conn = PgConnection::establish(&database_url)
				.wrap_err("Failed to connect to database for migrations")?;
			conn.run_pending_migrations(MIGRATIONS)
				.map_err(|e| eyre::eyre!("Migration failed: {e}"))?;
			tracing::info!("Database migrations up to date");
			Ok::<_, eyre::Report>(())
		})
		.await
		.map_err(|e| eyre::eyre!("Migration thread panicked: {e}"))??;
	}

	// Warm the connection pool so the first request isn't penalised by lazy
	// connection setup, and to fail early if the database is unreachable.
	let _conn = pool.get().await.wrap_err("Failed to connect to database")?;
	drop(_conn);
	tracing::info!("Connection pool warmed up");

	// HTTP server
	// -----------
	let app = routes::router(pool);

	let listener = tokio::net::TcpListener::bind(&config.listen_socket)
		.await
		.wrap_err("Failed to bind address")?;

	tracing::info!("Listening on {}", config.listen_socket);
	axum::serve(listener, app).await.wrap_err("Server error")?;

	Ok(())
}
