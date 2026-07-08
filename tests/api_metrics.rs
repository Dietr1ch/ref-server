/// Integration tests for the /metrics API
///
/// Code,
/// - :/src/routes/mod.rs
/// Docs,
/// - :/docs/api/metrics.org
use axum::body::Body;
use axum::http::{Request, StatusCode};
use googletest::prelude::*;
use tower::ServiceExt;

mod common;

#[tokio::test]
async fn metrics_returns_200() {
	let app = common::test_app().await;

	let response = app
		.oneshot(
			Request::builder()
				.uri("/metrics")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_that!(response.status(), eq(StatusCode::OK));
}

#[tokio::test]
async fn metrics_contains_expected_entries() {
	let app = common::test_app().await;

	let response = app
		.oneshot(
			Request::builder()
				.uri("/metrics")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
		.await
		.unwrap();
	let body = String::from_utf8(body_bytes.to_vec()).unwrap();

	assert_that!(body, contains_substring("axum_http_requests_total"));
	assert_that!(
		body,
		contains_substring("axum_http_requests_duration_seconds")
	);
	assert_that!(body, contains_substring("axum_http_requests_pending"));
}
