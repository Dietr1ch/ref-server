/// Implementation for the /health API
///
/// Docs,
/// - :/docs/api/health.org
///
/// Relevant integration tests,
/// - :/tests/api_health.rs
use axum::Json;
use serde_json::json;

/// Liveness / readiness check.
pub async fn health() -> Json<serde_json::Value> {
	Json(json!({ "status": "ok" }))
}

#[cfg(test)]
mod tests {
	use super::*;
	use googletest::prelude::*;

	#[tokio::test]
	async fn health_returns_ok() {
		let response = health().await;

		assert_that!(response.0, eq(&json!({"status": "ok"})));
	}

	#[tokio::test]
	async fn health_status_is_string_ok() {
		let response = health().await;

		assert_that!(
			response.0.get("status").and_then(|v| v.as_str()),
			some(eq("ok"))
		);
	}
}
