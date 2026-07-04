use std::sync::LazyLock;

use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{MigrationHarness, embed_migrations};
use eyre::Context;
use tracing_subscriber::EnvFilter;

use demo_server::routes;

/// Embedded Diesel migrations (run automatically on startup).
const MIGRATIONS: diesel_migrations::EmbeddedMigrations = embed_migrations!("migrations");

/// Default log filter: info-level for our crate and tower-http.
static LOG_FILTER: LazyLock<String> = LazyLock::new(|| {
	std::env::var("RUST_LOG").unwrap_or_else(|_| "demo_server=info,tower_http=info".into())
});

#[tokio::main]
async fn main() -> eyre::Result<()> {
	// Load .env file (sibling to Cargo.toml)
	dotenvy::dotenv().ok();

	// Initialise structured logging
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::new(&*LOG_FILTER))
		.init();

	// Database
	// --------
	let database_url = std::env::var("DATABASE_URL").wrap_err("DATABASE_URL must be set")?;

	let manager = ConnectionManager::<PgConnection>::new(&database_url);
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

	let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into());
	let listener = tokio::net::TcpListener::bind(&bind_addr)
		.await
		.wrap_err("Failed to bind address")?;

	tracing::info!("Listening on {bind_addr}");
	axum::serve(listener, app).await.wrap_err("Server error")?;

	Ok(())
}
