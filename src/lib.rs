pub mod models;
pub mod routes;
pub mod schema;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

/// Type alias for our PostgreSQL connection pool.
pub type DbPool = Pool<ConnectionManager<PgConnection>>;
