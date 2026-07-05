pub mod models;
pub mod routes;
pub mod error;
pub mod schema;

use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::AsyncPgConnection;

/// Type alias for our PostgreSQL connection pool.
pub type DbPool = Pool<AsyncPgConnection>;
