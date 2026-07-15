use diesel::Connection;
use diesel::pg::PgConnection;
use diesel_async::AsyncConnection;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_migrations::{MigrationHarness, embed_migrations};

use ref_server::routes;

const MIGRATIONS: diesel_migrations::EmbeddedMigrations = embed_migrations!("migrations");

/// Build a [`axum_test::TestServer`] backed by a dedicated connection pool.
///
/// Each call runs pending migrations, starts a test transaction on a single
/// pooled connection, and returns a server whose handlers share that
/// connection.  When the returned server is dropped the pool is dropped too,
/// closing the connection — PostgreSQL rolls back the open transaction,
/// so no cleanup is needed.
pub async fn test_server() -> axum_test::TestServer {
	build_test_server(build_pool().await, vec![])
}

/// Same as [`test_server`] but with the given CORS allowed origins.
///
/// Pass a list like `["https://mypage.com"]` to test CORS behaviour, or
/// `["*"]` for any origin.
#[allow(dead_code)]
pub async fn test_server_with_cors(origins: Vec<&str>) -> axum_test::TestServer {
	let pool = build_pool().await;
	build_test_server(pool, origins.into_iter().map(String::from).collect())
}

async fn build_pool() -> Pool<AsyncPgConnection> {
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

	// Start a test transaction on the single pooled connection.
	{
		let mut conn = pool
			.get()
			.await
			.expect("Failed to get connection for test transaction");
		conn.begin_test_transaction()
			.await
			.expect("Failed to begin test transaction");
	}

	pool
}

fn build_test_server(pool: Pool<AsyncPgConnection>, origins: Vec<String>) -> axum_test::TestServer {
	axum_test::TestServer::new(routes::router(pool, origins))
}
