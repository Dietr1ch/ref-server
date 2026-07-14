/// Integration tests for the /users/{id}/posts API
///
/// Code,
/// - :/src/routes/posts.rs
use axum::http::StatusCode;
use googletest::prelude::*;
use googletest_json_serde::json as j;
use uuid::Uuid;

mod common;

#[gtest]
#[tokio::test]
async fn list_posts_empty_returns_empty_array() {
	let server = common::test_server().await;

	// Create a user first
	let response = server
		.post("/users")
		.json(&serde_json::json!({
			"name": "Alice",
			"email": "alice@example.com",
		}))
		.await;
	let user_id: Uuid = response
		.json::<serde_json::Value>()
		.get("id")
		.and_then(|v| v.as_str())
		.and_then(|s| s.parse().ok())
		.expect("Created user should have a valid UUID `id`");

	let response = server.get(&format!("/users/{user_id}/posts")).await;

	expect_that!(response.status_code(), eq(StatusCode::OK));
	expect_that!(
		response.json::<serde_json::Value>(),
		eq(&serde_json::json!([]))
	);
}

#[gtest]
#[tokio::test]
async fn create_and_list_posts() {
	let server = common::test_server().await;

	// Create a user
	let response = server
		.post("/users")
		.json(&serde_json::json!({
			"name": "Bob",
			"email": "bob@example.com",
		}))
		.await;
	let user_id: Uuid = response
		.json::<serde_json::Value>()
		.get("id")
		.and_then(|v| v.as_str())
		.and_then(|s| s.parse().ok())
		.expect("Created user should have a valid UUID `id`");

	// Create a post
	let response = server
		.post(&format!("/users/{user_id}/posts"))
		.json(&serde_json::json!({
			"title": "Hello World",
			"body": "My first post",
		}))
		.await;

	expect_that!(response.status_code(), eq(StatusCode::CREATED));

	let post_id: Uuid = response
		.json::<serde_json::Value>()
		.get("id")
		.and_then(|v| v.as_str())
		.and_then(|s| s.parse().ok())
		.expect("Created post should have a valid UUID `id`");

	// List posts
	let response = server.get(&format!("/users/{user_id}/posts")).await;

	expect_that!(response.status_code(), eq(StatusCode::OK));
	expect_that!(
		response.json::<serde_json::Value>(),
		j::elements_are![
			// Single Post
			j::pat!({
				"id": eq(post_id.to_string()),
				"user_id": eq(user_id.to_string()),
				"title": eq("Hello World"),
				"body": eq("My first post"),
				"created_at": j::is_non_empty_string(),
				..
			})
		]
	);
}

#[gtest]
#[tokio::test]
async fn create_post_without_body_defaults_to_empty() {
	let server = common::test_server().await;

	let response = server
		.post("/users")
		.json(&serde_json::json!({
			"name": "Carol",
			"email": "carol@example.com",
		}))
		.await;
	let user_id: Uuid = response
		.json::<serde_json::Value>()
		.get("id")
		.and_then(|v| v.as_str())
		.and_then(|s| s.parse().ok())
		.expect("Created user should have a valid UUID `id`");

	// Create a post without body
	let response = server
		.post(&format!("/users/{user_id}/posts"))
		.json(&serde_json::json!({
			"title": "No body post",
		}))
		.await;

	expect_that!(response.status_code(), eq(StatusCode::CREATED));
}

#[gtest]
#[tokio::test]
async fn create_post_nonexistent_user_returns_404() {
	let server = common::test_server().await;

	let response = server
		.post(&format!("/users/{}/posts", Uuid::nil()))
		.json(&serde_json::json!({
			"title": "Orphan post",
			"body": "This should fail",
		}))
		.await;

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
async fn list_posts_nonexistent_user_returns_empty() {
	let server = common::test_server().await;

	// Listing posts for a nonexistent user returns an empty array (no FK constraint on SELECT)
	let response = server.get(&format!("/users/{}/posts", Uuid::nil())).await;

	expect_that!(response.status_code(), eq(StatusCode::OK));
	expect_that!(
		response.json::<serde_json::Value>(),
		eq(&serde_json::json!([]))
	);
}
