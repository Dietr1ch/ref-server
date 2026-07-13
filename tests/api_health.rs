/// Integration tests for the /health API
///
/// Code,
/// - :/src/routes/health.rs
/// Docs,
/// - :/docs/api/health.org
use axum::body::Body;
use axum::http::{Request, StatusCode};
use googletest::prelude::*;
use tower::ServiceExt;

mod common;

#[gtest]
#[tokio::test]
async fn health_returns_200() {
	let app = common::test_app().await;

	let response = app
		.oneshot(
			Request::builder()
				.uri("/health")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	expect_that!(response.status(), eq(StatusCode::OK));

	let json = common::response_json(response).await;
	expect_that!(json, eq(&serde_json::json!({"status": "ok"})));
}
