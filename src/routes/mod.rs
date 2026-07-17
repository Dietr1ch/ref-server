mod health;
mod posts;
mod users;

use axum::Router;
use axum::http::{HeaderValue, header};
use axum::routing::get;
use axum_prometheus::PrometheusMetricLayer;
use tower::ServiceBuilder;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::app;

// Routing
// =======

/// Build the axum router with all routes and shared state.
///
/// `cors_allow_origins` controls the [`Access-Control-Allow-Origin`] response
/// header.  Pass an empty vec to disable CORS entirely, a single origin like
/// `["https://mypage.com"]`, or `["*"]` to allow any origin.
pub fn router(pool: app::DbPool, cors_allow_origins: Vec<String>) -> Router {
	let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

	let mut router = Router::new()
		.route("/health", get(health::health))
		.route("/users", get(users::list_users).post(users::create_user))
		.route("/users/{id}", get(users::get_user).patch(users::patch_user))
		.nest(
			"/users/{user_id}",
			Router::new().route("/posts", get(posts::list_posts).post(posts::create_post)),
		)
		.layer(
			ServiceBuilder::new()
				.layer(axum::middleware::from_fn(crate::middleware::request_timer)),
		)
		.layer(prometheus_layer)
		.route("/metrics", get(|| async move { metric_handle.render() }));

	if let Some(cors) = build_cors(cors_allow_origins) {
		router = router.layer(cors);
	}

	router.with_state(pool)
}

/// Build a [`CorsLayer`] from the list of allowed origins, or return [`None`]
/// when the list is empty.
fn build_cors(origins: Vec<String>) -> Option<CorsLayer> {
	if origins.is_empty() {
		return None;
	}

	let allow_origin = if origins.iter().any(|o| o == "*") {
		AllowOrigin::any()
	} else {
		let parsed: Vec<HeaderValue> = origins
			.into_iter()
			.filter_map(|o| match o.parse::<HeaderValue>() {
				Ok(v) => Some(v),
				Err(e) => {
					tracing::warn!("Skipping invalid CORS origin {o:?}: {e}");
					None
				}
			})
			.collect();

		if parsed.is_empty() {
			tracing::warn!("No valid CORS origins configured — CORS disabled");
			return None;
		}

		AllowOrigin::list(parsed)
	};

	Some(
		CorsLayer::new()
			.allow_origin(allow_origin)
			.allow_methods([
				axum::http::Method::GET,
				axum::http::Method::POST,
				axum::http::Method::PATCH,
				axum::http::Method::DELETE,
			])
			.allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]),
	)
}
