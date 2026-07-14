/// Integration tests for the /health API
///
/// Code,
/// - :/src/routes/health.rs
/// Docs,
/// - :/docs/api/health.org
use axum::http::StatusCode;
use googletest::prelude::*;

mod common;

#[gtest]
#[tokio::test]
async fn health_returns_200() {
	let server = common::test_server().await;

	let response = server.get("/health").await;

	expect_that!(response.status_code(), eq(StatusCode::OK));
	expect_that!(
		response.json::<serde_json::Value>(),
		eq(&serde_json::json!({"status": "ok"}))
	);
}
