use axum::{
	Json, Router,
	extract::{Path, State},
	http::StatusCode,
	routing::get,
};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde_json::json;
use uuid::Uuid;

use crate::models::{NewUser, User};

/// Type alias for our PostgreSQL connection pool.
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

/// Build the axum router with all routes and shared state.
pub fn router(pool: DbPool) -> Router {
	Router::new()
		.route("/health", get(health))
		.route("/users", get(list_users).post(create_user))
		.route("/users/{id}", get(get_user))
		.with_state(pool)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Liveness / readiness check.
async fn health() -> Json<serde_json::Value> {
	Json(json!({ "status": "ok" }))
}

/// GET /users — list all users.
async fn list_users(State(pool): State<DbPool>) -> Result<Json<Vec<User>>, AppError> {
	use crate::schema::users::dsl::*;

	let mut conn = pool.get().map_err(AppError::internal)?;
	let results = users
		.select(User::as_select())
		.order(created_at.asc())
		.load(&mut conn)
		.map_err(AppError::internal)?;

	Ok(Json(results))
}

/// POST /users — create a new user.
async fn create_user(
	State(pool): State<DbPool>,
	Json(new_user): Json<NewUser>,
) -> Result<(StatusCode, Json<User>), AppError> {
	use crate::schema::users::dsl::*;

	let mut conn = pool.get().map_err(AppError::internal)?;
	let user = diesel::insert_into(users)
		.values(&new_user)
		.returning(User::as_returning())
		.get_result(&mut conn)
		.map_err(|e| match e {
			diesel::result::Error::DatabaseError(
				diesel::result::DatabaseErrorKind::UniqueViolation,
				_,
			) => AppError::conflict("A user with this email already exists"),
			other => AppError::internal(other),
		})?;

	Ok((StatusCode::CREATED, Json(user)))
}

/// GET /users/{id} — fetch a single user by UUID.
async fn get_user(
	State(pool): State<DbPool>,
	Path(user_id): Path<Uuid>,
) -> Result<Json<User>, AppError> {
	use crate::schema::users::dsl::*;

	let mut conn = pool.get().map_err(AppError::internal)?;
	let user = users
		.find(user_id)
		.select(User::as_select())
		.first(&mut conn)
		.map_err(|e| match e {
			diesel::result::Error::NotFound => {
				AppError::not_found(format!("User {user_id} not found"))
			}
			other => AppError::internal(other),
		})?;

	Ok(Json(user))
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

/// Unified application error type that produces JSON error responses.
struct AppError {
	status: StatusCode,
	message: String,
}

impl AppError {
	fn internal<E: std::fmt::Debug>(err: E) -> Self {
		Self {
			status: StatusCode::INTERNAL_SERVER_ERROR,
			message: format!("{err:?}"),
		}
	}

	fn not_found(msg: impl Into<String>) -> Self {
		Self {
			status: StatusCode::NOT_FOUND,
			message: msg.into(),
		}
	}

	fn conflict(msg: impl Into<String>) -> Self {
		Self {
			status: StatusCode::CONFLICT,
			message: msg.into(),
		}
	}
}

/// Allow axum to convert our error into an HTTP response.
impl axum::response::IntoResponse for AppError {
	fn into_response(self) -> axum::response::Response {
		let body = json!({ "error": self.message });
		(self.status, Json(body)).into_response()
	}
}
