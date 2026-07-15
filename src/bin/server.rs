use std::time::Duration;

use clap::Parser;
use diesel::Connection;
use diesel::pg::PgConnection;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
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

	// CORS
	/// Allowed origins for CORS (repeatable, or use `*` for any).
	/// When empty, no CORS headers are sent.
	#[arg(long, env = "CORS_ALLOW_ORIGINS", value_delimiter = ',')]
	cors_allow_origins: Vec<String>,
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
	tracing::info!("Initialising DB connections...");
	let connection_config =
		AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.database_url);

	tracing::debug!("Initialising DB connection Pool...");
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
		tracing::debug!(" Checking for pending migrations...");
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
	{
		use diesel::dsl::count_star;
		use diesel::query_dsl::methods::SelectDsl;

		tracing::info!("Warming up DB connection...");
		let mut conn = pool.get().await.wrap_err("Failed to connect to database")?;

		let count: i64 = ref_server::schema::users::table
			.select(count_star())
			.first(&mut conn)
			.await
			.wrap_err("Warm-up query failed")?;

		tracing::info!("Connection pool warmed up; {count} users in database");
	}

	// HTTP server
	// -----------
	let app = routes::router(pool, config.cors_allow_origins);

	let listener = tokio::net::TcpListener::bind(&config.listen_socket)
		.await
		.wrap_err("Failed to bind address")?;

	tracing::info!("Listening on {}", config.listen_socket);
	axum::serve(listener, app).await.wrap_err("Server error")?;

	Ok(())
}
