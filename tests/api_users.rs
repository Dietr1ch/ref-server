/// Integration tests for the /users API
///
/// Code,
/// - :/src/routes/users.rs
/// Docs,
/// - :/docs/api/users.org
use axum::http::StatusCode;
use googletest::prelude::*;
use googletest_json_serde::json as j;
use uuid::Uuid;

mod common;

#[gtest]
#[tokio::test]
async fn list_users_empty_returns_empty_array() {
	let server = common::test_server().await;

	let response = server.get("/users").await;

	expect_that!(response.status_code(), eq(StatusCode::OK));
	expect_that!(
		response.json::<serde_json::Value>(),
		eq(&serde_json::json!([]))
	);
}

#[gtest]
#[tokio::test]
async fn create_and_get_user() {
	let server = common::test_server().await;

	// Create a user
	let response = server
		.post("/users")
		.json(&serde_json::json!({
			"name": "Alice",
			"email": "alice@example.com",
		}))
		.await;

	expect_that!(response.status_code(), eq(StatusCode::CREATED));

	let user_id: Uuid = response
		.json::<serde_json::Value>()
		.get("id")
		.and_then(|v| v.as_str())
		.and_then(|s| s.parse().ok())
		.expect("Created user should have a valid UUID `id`");
	// Create response only returns the id; name/email are not echoed back.

	// Get user by ID
	let response = server.get(&format!("/users/{user_id}")).await;

	expect_that!(response.status_code(), eq(StatusCode::OK));
	expect_that!(
		&response.json::<serde_json::Value>(),
		j::pat!({
			"name": eq("Alice"),
			"email": eq("alice@example.com"),
			"created_at": j::is_non_empty_string(),
			..
		})
	);
}

#[gtest]
#[tokio::test]
async fn list_users_with_fields() {
	let server = common::test_server().await;

	// Create a user so we have something to list.
	let response = server
		.post("/users")
		.json(&serde_json::json!({
			"name": "Bob",
			"email": "bob-fields@example.com",
		}))
		.await;

	expect_that!(response.status_code(), eq(StatusCode::CREATED));
	let user_id = response
		.json::<serde_json::Value>()
		.get("id")
		.and_then(|v| v.as_str())
		.expect("Created user should have a valid UUID `id`")
		.to_owned();

	// Only request the `id` field
	let response = server.get("/users?fields=id").await;
	expect_that!(response.status_code(), eq(StatusCode::OK));
	expect_that!(
		response.json::<serde_json::Value>(),
		eq(&serde_json::json!([{"id": user_id}]))
	);

	// Request `id` and `name`
	let response = server.get("/users?fields=id,name").await;
	expect_that!(response.status_code(), eq(StatusCode::OK));
	expect_that!(
		response.json::<serde_json::Value>(),
		eq(&serde_json::json!([{"id": user_id, "name": "Bob"}]))
	);

	// Request an unknown field (id is always included)
	let response = server.get("/users?fields=nonexistent").await;
	expect_that!(response.status_code(), eq(StatusCode::OK));
	expect_that!(
		response.json::<serde_json::Value>(),
		eq(&serde_json::json!([{"id": user_id}]))
	);
}

#[gtest]
#[tokio::test]
async fn get_nonexistent_user_returns_404() {
	let server = common::test_server().await;

	let response = server.get(&format!("/users/{}", Uuid::nil())).await;

	expect_that!(response.status_code(), eq(StatusCode::NOT_FOUND));
	expect_that!(
		&response.json::<serde_json::Value>(),
		j::pat!({
			"error": contains_substring("not found"),
			..
		})
	);
}

#[gtest]
#[tokio::test]
async fn create_duplicate_email_returns_409() {
	let server = common::test_server().await;

	let body = serde_json::json!({
		"name": "Bob",
		"email": "bob@example.com",
	});

	// First create — should succeed
	let response = server.post("/users").json(&body).await;
	expect_that!(response.status_code(), eq(StatusCode::CREATED));

	// Second create with same email — should conflict
	let response = server.post("/users").json(&body).await;
	expect_that!(response.status_code(), eq(StatusCode::CONFLICT));
	expect_that!(
		&response.json::<serde_json::Value>(),
		j::pat!({
			"error": contains_substring("already exists"),
			..
		})
	);
}
