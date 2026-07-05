mod health;
mod user;

use axum::Json;
use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;
use serde_json::json;

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

// Error handling
// ==============

/// Unified application error type that produces JSON error responses.
struct AppError {
	status: StatusCode,
	message: String,
}

impl AppError {
	fn internal<E: std::fmt::Debug>(err: E) -> Self {
		Self {
			status: StatusCode::INTERNAL_SERVER_ERROR,
			message: format!("{err:?}"),
		}
	}

	fn not_found(msg: impl Into<String>) -> Self {
		Self {
			status: StatusCode::NOT_FOUND,
			message: msg.into(),
		}
	}

	fn conflict(msg: impl Into<String>) -> Self {
		Self {
			status: StatusCode::CONFLICT,
			message: msg.into(),
		}
	}
}

/// Allow axum to convert our error into an HTTP response.
impl axum::response::IntoResponse for AppError {
	fn into_response(self) -> axum::response::Response {
		let body = json!({ "error": self.message });
		(self.status, Json(body)).into_response()
	}
}
