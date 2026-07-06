use axum::body::Body;
use axum::http::{Request, StatusCode};
use googletest::prelude::*;
use tower::ServiceExt;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn list_users_empty_returns_empty_array() {
	let app = common::test_app().await;

	let response = app
		.oneshot(Request::get("/users").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_that!(response.status(), eq(StatusCode::OK));

	let list: serde_json::Value = common::response_json(response).await;
	assert_that!(list, eq(&serde_json::json!([])));
}

#[tokio::test]
async fn create_and_list_and_get_user() {
	let app = common::test_app().await;

	// Create a user
	let create_body = serde_json::json!({
		"name": "Alice",
		"email": "alice@example.com",
	});

	let response = app
		.clone()
		.oneshot(
			Request::post("/users")
				.header("Content-Type", "application/json")
				.body(Body::from(serde_json::to_vec(&create_body).unwrap()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_that!(response.status(), eq(StatusCode::CREATED));

	let created: serde_json::Value = common::response_json(response).await;
	let user_id: Uuid = created
		.get("id")
		.and_then(|v| v.as_str())
		.and_then(|s| s.parse().ok())
		.expect("Created user should have a valid UUID `id`");
	assert_that!(
		created.get("name").and_then(|v| v.as_str()),
		some(eq("Alice"))
	);
	assert_that!(
		created.get("email").and_then(|v| v.as_str()),
		some(eq("alice@example.com"))
	);

	// List users
	let list_response = app
		.clone()
		.oneshot(Request::get("/users").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_that!(list_response.status(), eq(StatusCode::OK));
	let list: serde_json::Value = common::response_json(list_response).await;
	assert_that!(list, eq(&serde_json::json!([{"id": user_id.to_string()}])));

	// Get user by ID
	let get_response = app
		.clone()
		.oneshot(
			Request::get(format!("/users/{user_id}"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_that!(get_response.status(), eq(StatusCode::OK));
	let fetched: serde_json::Value = common::response_json(get_response).await;
	assert_that!(fetched, eq(&created));
}

#[tokio::test]
async fn list_users_with_fields() {
	let app = common::test_app().await;

	// Create a user so we have something to list.
	let create_body = serde_json::json!({
		"name": "Bob",
		"email": "bob-fields@example.com",
	});
	let response = app
		.clone()
		.oneshot(
			Request::post("/users")
				.header("Content-Type", "application/json")
				.body(Body::from(serde_json::to_vec(&create_body).unwrap()))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_that!(response.status(), eq(StatusCode::CREATED));
	let created: serde_json::Value = common::response_json(response).await;
	let user_id = created
		.get("id")
		.and_then(|v| v.as_str())
		.unwrap()
		.to_owned();

	// Only request the `id` field
	let r = app
		.clone()
		.oneshot(
			Request::get("/users?fields=id")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_that!(r.status(), eq(StatusCode::OK));
	let list: serde_json::Value = common::response_json(r).await;
	assert_that!(list, eq(&serde_json::json!([{"id": user_id}])));

	// Request `id` and `name`
	let r = app
		.clone()
		.oneshot(
			Request::get("/users?fields=id,name")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_that!(r.status(), eq(StatusCode::OK));
	let list: serde_json::Value = common::response_json(r).await;
	assert_that!(
		list,
		eq(&serde_json::json!([{"id": user_id, "name": "Bob"}]))
	);

	// Request an unknown field (id is always included)
	let r = app
		.clone()
		.oneshot(
			Request::get("/users?fields=nonexistent")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_that!(r.status(), eq(StatusCode::OK));
	let list: serde_json::Value = common::response_json(r).await;
	assert_that!(list, eq(&serde_json::json!([{"id": user_id}])));
}

#[tokio::test]
async fn get_nonexistent_user_returns_404() {
	let app = common::test_app().await;

	let nonexistent = Uuid::nil();
	let response = app
		.oneshot(
			Request::get(format!("/users/{nonexistent}"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_that!(response.status(), eq(StatusCode::NOT_FOUND));

	let json = common::response_json(response).await;
	assert_that!(
		json.get("error").and_then(|v| v.as_str()),
		some(contains_substring("not found"))
	);
}

#[tokio::test]
async fn create_duplicate_email_returns_409() {
	let app = common::test_app().await;

	let body = serde_json::json!({
		"name": "Bob",
		"email": "bob@example.com",
	});
	let bytes = serde_json::to_vec(&body).unwrap();

	// First create — should succeed
	let r1 = app
		.clone()
		.oneshot(
			Request::post("/users")
				.header("Content-Type", "application/json")
				.body(Body::from(bytes.clone()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_that!(r1.status(), eq(StatusCode::CREATED));

	// Second create with same email — should conflict
	let r2 = app
		.clone()
		.oneshot(
			Request::post("/users")
				.header("Content-Type", "application/json")
				.body(Body::from(bytes))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_that!(r2.status(), eq(StatusCode::CONFLICT));

	let json = common::response_json(r2).await;
	assert_that!(
		json.get("error").and_then(|v| v.as_str()),
		some(contains_substring("already exists"))
	);
}
