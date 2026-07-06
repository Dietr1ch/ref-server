use diesel::Connection;
use diesel::pg::PgConnection;
use diesel_async::AsyncConnection;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_migrations::{MigrationHarness, embed_migrations};

use ref_server::DbPool;
use ref_server::routes;

const MIGRATIONS: diesel_migrations::EmbeddedMigrations = embed_migrations!("migrations");

/// Build a test pool and run pending migrations.
///
/// The pool has a single connection with an active test transaction that is
/// rolled back when the pool is dropped — so each test starts with a clean
/// database and no cleanup is needed.
pub async fn test_setup() -> DbPool {
	let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");

	// Run migrations using sync libpq (matches server startup behaviour).
	let url = database_url.clone();
	tokio::task::spawn_blocking(move || {
		let mut conn =
			PgConnection::establish(&url).expect("Failed to connect for migrations in test");
		conn.run_pending_migrations(MIGRATIONS)
			.expect("Migrations failed in test");
	})
	.await
	.expect("Migration thread panicked");

	let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
	let pool = Pool::builder()
		.max_size(1)
		.min_idle(Some(1))
		.connection_timeout(std::time::Duration::from_secs(5))
		.build(config)
		.await
		.expect("Failed to build test pool");

	// Begin a test transaction on the single pooled connection.  All handler
	// requests during this test will use the same connection — still inside
	// this transaction — and PostgreSQL will roll it back when the pool drops.
	let mut conn = pool
		.get()
		.await
		.expect("Failed to get connection for test transaction");
	conn.begin_test_transaction()
		.await
		.expect("Failed to begin test transaction");
	drop(conn);

	pool
}

/// Build the full axum router backed by a real database.
pub async fn test_app() -> axum::Router {
	let pool = test_setup().await;
	routes::router(pool)
}

/// Helper: parse a response body into a JSON Value.
pub async fn response_json(response: axum::response::Response) -> serde_json::Value {
	let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
		.await
		.unwrap();
	serde_json::from_slice(&body_bytes).unwrap()
}
