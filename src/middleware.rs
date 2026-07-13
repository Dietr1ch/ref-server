use std::time::Instant;

use axum::{extract::Request, middleware::Next, response::Response};

/// Middleware that logs the HTTP method, URI, and elapsed time for every request.
pub async fn request_timer(request: Request, next: Next) -> Response {
	let start = Instant::now();
	let method = request.method().clone();
	let uri = request.uri().clone();

	let response = next.run(request).await;

	let elapsed = start.elapsed();
	tracing::debug!("{} {} took {:?}", method, uri, elapsed);

	response
}

#[cfg(test)]
mod tests {
	use super::*;
	use axum::{Json, Router, routing::get};
	use googletest::prelude::*;
	use tower::ServiceExt;

	#[gtest]
	#[tokio::test]
	async fn timer_passthrough_ok_response() {
		let app = Router::new()
			.route(
				"/ping",
				get(|| async { Json(serde_json::json!({"ok": true})) }),
			)
			.layer(axum::middleware::from_fn(request_timer));

		let response = app
			.oneshot(
				axum::http::Request::builder()
					.uri("/ping")
					.body(String::new())
					.unwrap(),
			)
			.await
			.unwrap();

		expect_that!(response.status(), eq(200));
	}

	#[gtest]
	#[tokio::test]
	async fn timer_passthrough_404_response() {
		let app = Router::new()
			.route(
				"/ping",
				get(|| async { Json(serde_json::json!({"ok": true})) }),
			)
			.layer(axum::middleware::from_fn(request_timer));

		let response = app
			.oneshot(
				axum::http::Request::builder()
					.uri("/nonexistent")
					.body(String::new())
					.unwrap(),
			)
			.await
			.unwrap();

		expect_that!(response.status(), eq(404));
	}
}
