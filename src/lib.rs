pub mod error;
pub mod models;
pub mod routes;
pub mod schema;

use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::bb8::Pool;

/// Type alias for our PostgreSQL connection pool.
pub type DbPool = Pool<AsyncPgConnection>;
