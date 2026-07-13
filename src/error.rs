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

#[cfg(test)]
mod tests {
	use super::*;
	use axum::response::IntoResponse;
	use googletest::prelude::*;

	async fn body_json(response: axum::response::Response) -> serde_json::Value {
		let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
			.await
			.unwrap();
		serde_json::from_slice(&body_bytes).unwrap()
	}

	#[gtest]
	#[tokio::test]
	async fn internal_error_returns_500() {
		let response = AppError::internal("db connection failed").into_response();

		expect_that!(response.status(), eq(StatusCode::INTERNAL_SERVER_ERROR));
	}

	#[gtest]
	#[tokio::test]
	async fn internal_error_includes_message() {
		let response = AppError::internal("db connection failed").into_response();
		let json = body_json(response).await;

		expect_that!(
			json.get("error").and_then(|v| v.as_str()),
			some(contains_substring("db connection failed"))
		);
	}

	#[gtest]
	#[tokio::test]
	async fn not_found_returns_404() {
		let response = AppError::not_found("User 123 not found").into_response();

		expect_that!(response.status(), eq(StatusCode::NOT_FOUND));
	}

	#[gtest]
	#[tokio::test]
	async fn not_found_includes_message() {
		let response = AppError::not_found("User 123 not found").into_response();
		let json = body_json(response).await;

		expect_that!(
			json.get("error").and_then(|v| v.as_str()),
			some(eq("User 123 not found"))
		);
	}

	#[gtest]
	#[tokio::test]
	async fn conflict_returns_409() {
		let response = AppError::conflict("email already taken").into_response();

		expect_eq!(response.status(), StatusCode::CONFLICT);
	}

	#[gtest]
	#[tokio::test]
	async fn conflict_includes_message() {
		let response = AppError::conflict("email already taken").into_response();
		let json = body_json(response).await;

		expect_that!(
			json.get("error").and_then(|v| v.as_str()),
			some(eq("email already taken"))
		);
	}
}
