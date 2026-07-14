/// Integration tests for the /metrics API
///
/// Code,
/// - :/src/routes/mod.rs
/// Docs,
/// - :/docs/api/metrics.org
use axum::http::StatusCode;
use googletest::prelude::*;

mod common;

#[gtest]
#[tokio::test]
async fn metrics_returns_200() {
	let server = common::test_server().await;

	let response = server.get("/metrics").await;

	expect_that!(response.status_code(), eq(StatusCode::OK));
}

#[gtest]
#[tokio::test]
async fn metrics_contains_expected_entries() {
	let server = common::test_server().await;

	// NOTE: Request /health so there's data for the following /metrics request
	server.get("/health").await;

	let response = server.get("/metrics").await;
	let body = response.text();

	expect_that!(body, contains_substring("axum_http_requests_total"));
	expect_that!(
		body,
		contains_substring("axum_http_requests_duration_seconds")
	);
	expect_that!(body, contains_substring("axum_http_requests_pending"));
}
