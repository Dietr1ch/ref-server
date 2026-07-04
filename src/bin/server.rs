use clap::Parser;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{MigrationHarness, embed_migrations};
use eyre::Context;
use tracing_subscriber::EnvFilter;

use demo_server::routes;

/// Embedded Diesel migrations (run automatically on startup).
const MIGRATIONS: diesel_migrations::EmbeddedMigrations = embed_migrations!("migrations");

/// Configuration read from environment variables.
#[derive(Parser)]
#[command(
	name = "demo-server",
	version,
	about = "axum + diesel + PostgreSQL demo"
)]
struct Config {
	/// PostgreSQL connection string.
	#[arg(env = "DATABASE_URL")]
	database_url: String,

	/// Socket address (host:port) to listen on.
	#[arg(env = "LISTEN_SOCKET", default_value = "0.0.0.0:3000")]
	listen_socket: String,

	/// Tracing/logging filter.
	#[arg(env = "RUST_LOG", default_value = "demo_server=info,tower_http=info")]
	rust_log: String,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let config = Config::parse();

	// Structured logging
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::new(&config.rust_log))
		.init();

	// Database
	// --------
	let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
	let pool = Pool::builder()
		.build(manager)
		.wrap_err("Failed to build connection pool")?;

	// Run pending migrations at startup
	{
		let mut conn = pool
			.get()
			.wrap_err("Failed to get connection for migrations")?;
		conn.run_pending_migrations(MIGRATIONS)
			.map_err(|e| eyre::eyre!("Migration failed: {e}"))?;
		tracing::info!("Database migrations up to date");
	}

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
