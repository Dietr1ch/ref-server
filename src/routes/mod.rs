mod health;
mod user;

use axum::Router;
use axum::routing::get;

use crate::DbPool;

// Routing
// =======

/// Build the axum router with all routes and shared state.
pub fn router(pool: DbPool) -> Router {
	Router::new()
		.route("/health", get(health::health))
		.route("/users", get(user::list_users).post(user::create_user))
		.route("/users/{id}", get(user::get_user))
		.with_state(pool)
}
