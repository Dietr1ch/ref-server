mod health;
mod posts;
mod users;

use axum::Router;
use axum::routing::get;
use axum_prometheus::PrometheusMetricLayer;
use tower::ServiceBuilder;

use crate::app;

// Routing
// =======

/// Build the axum router with all routes and shared state.
pub fn router(pool: app::DbPool) -> Router {
	let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

	Router::new()
		.route("/health", get(health::health))
		.route("/users", get(users::list_users).post(users::create_user))
		.route("/users/{id}", get(users::get_user))
		.nest(
			"/users/{user_id}",
			Router::new().route("/posts", get(posts::list_posts).post(posts::create_post)),
		)
		.layer(
			ServiceBuilder::new()
				.layer(axum::middleware::from_fn(crate::middleware::request_timer)),
		)
		.layer(prometheus_layer)
		.route("/metrics", get(|| async move { metric_handle.render() }))
		.with_state(pool)
}
