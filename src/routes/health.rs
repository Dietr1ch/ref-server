use axum::Json;
use serde_json::json;

/// Liveness / readiness check.
pub async fn health() -> Json<serde_json::Value> {
	Json(json!({ "status": "ok" }))
}
