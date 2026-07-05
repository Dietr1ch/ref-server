use axum::Json;
use axum::http::StatusCode;
use serde_json::json;

/// Unified application error type that produces JSON error responses.
pub struct AppError {
	status: StatusCode,
	message: String,
}

impl AppError {
	pub fn internal<E: std::fmt::Debug>(err: E) -> Self {
		Self {
			status: StatusCode::INTERNAL_SERVER_ERROR,
			message: format!("{err:?}"),
		}
	}

	pub fn not_found(msg: impl Into<String>) -> Self {
		Self {
			status: StatusCode::NOT_FOUND,
			message: msg.into(),
		}
	}

	pub fn conflict(msg: impl Into<String>) -> Self {
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
