/// Integration tests for CORS headers.
///
/// Code,
/// - :/src/routes/mod.rs (CorsLayer setup)
use axum::http::{StatusCode, header};
use googletest::prelude::*;

mod common;

#[gtest]
#[tokio::test]
async fn cors_specific_origin_allowed() {
	let server = common::test_server_with_cors(vec!["https://mypage.com"]).await;

	let response = server
		.get("/health")
		.add_header(header::ORIGIN, "https://mypage.com")
		.await;

	expect_that!(response.status_code(), eq(StatusCode::OK));
	expect_that!(
		response.header(header::ACCESS_CONTROL_ALLOW_ORIGIN),
		eq("https://mypage.com")
	);
}

#[gtest]
#[tokio::test]
async fn cors_specific_origin_rejected() {
	let server = common::test_server_with_cors(vec!["https://mypage.com"]).await;

	let response = server
		.get("/health")
		.add_header(header::ORIGIN, "https://evil.com")
		.await;

	expect_that!(response.status_code(), eq(StatusCode::OK));
	expect_that!(
		response.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
		none()
	);
}

#[gtest]
#[tokio::test]
async fn cors_wildcard_allows_any_origin() {
	let server = common::test_server_with_cors(vec!["*"]).await;

	let response = server
		.get("/health")
		.add_header(header::ORIGIN, "https://anything.example.com")
		.await;

	expect_that!(response.status_code(), eq(StatusCode::OK));
	expect_that!(
		response.header(header::ACCESS_CONTROL_ALLOW_ORIGIN),
		eq("*")
	);
}

#[gtest]
#[tokio::test]
async fn cors_disabled_when_empty() {
	let server = common::test_server().await;

	let response = server
		.get("/health")
		.add_header(header::ORIGIN, "https://mypage.com")
		.await;

	expect_that!(response.status_code(), eq(StatusCode::OK));
	expect_that!(
		response.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
		none()
	);
}
