/// Implementation for the /users API
///
/// Docs,
/// - :/docs/api/users.org
///
/// Relevant integration tests,
/// - :/tests/api_users.rs
use axum::{
	Json,
	extract::{Path, Query, State},
	http::StatusCode,
};
use diesel::prelude::*;
use diesel::query_source::Column;
use diesel::sql_types::Text;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use uuid::Uuid;

use crate::DbPool;
use crate::error::AppError;
use crate::models::user::{NewUser, User};

/// Query parameters for the users collection endpoint.
#[derive(Debug, Default, Deserialize)]
pub struct UsersQuery {
	/// Comma-separated list of extra fields (beyond the mandatory `id`)
	/// to include in each user object.  When omitted, only `id` is returned.
	pub fields: Option<String>,
}

/// Columns in the `users` table that clients may request via `?fields`.
///
/// Each entry reads its SQL name from the corresponding Diesel column type,
/// so this list is always in sync with `diesel::table! { users ... }` in
/// `schema.rs` — no separate maintenance needed.
use crate::schema::users::columns as user_cols;
const COLUMNS: &[&str] = &[
	user_cols::id::NAME,
	user_cols::name::NAME,
	user_cols::email::NAME,
	user_cols::created_at::NAME,
];

/// Build a comma-separated SELECT clause from the requested fields.
///
/// `id` is always included; unknown column names are silently dropped.
fn select_clause(fields: Option<&str>) -> String {
	let mut cols: Vec<&str> = vec!["id"];

	if let Some(raw) = fields {
		for name in raw.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
			if name != "id" && COLUMNS.contains(&name) {
				cols.push(name);
			}
		}
	}

	tracing::debug!("Selecting columns: {cols:?}");
	cols.join(", ")
}

/// Helper struct for loading a single text value from a raw SQL query.
#[derive(QueryableByName)]
struct DataRow {
	#[diesel(sql_type = Text)]
	data: String,
}

/// GET /users — list all users.
///
/// The primary key (`id`) is always included.
/// Use `?fields` to request additional columns:
///
/// Examples:
///   GET /users                    → `[{id: …}, …]`
///   GET /users?fields=name,email  → `[{id: …, name: …, email: …}, …]`
///   GET /users?fields=name        → `[{id: …, name: …}, …]`
pub async fn list_users(
	State(pool): State<DbPool>,
	Query(params): Query<UsersQuery>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
	let columns = select_clause(params.fields.as_deref());

	let sql = format!(
		"SELECT COALESCE(jsonb_agg(to_jsonb(t)), '[]'::jsonb)::text AS data \
		 FROM (SELECT {columns} FROM users) t",
	);

	let mut conn = pool.get().await.map_err(AppError::internal)?;

	tracing::debug!("Querying: {sql:?}");
	let row: DataRow = diesel::sql_query(sql)
		.get_result(&mut conn)
		.await
		.map_err(|e| AppError::internal(format!("{e}")))?;

	let items: Vec<serde_json::Value> =
		serde_json::from_str(&row.data).map_err(|e| AppError::internal(format!("{e}")))?;

	Ok(Json(items))
}

/// POST /users — create a new user.
pub async fn create_user(
	State(pool): State<DbPool>,
	Json(new_user): Json<NewUser>,
) -> Result<(StatusCode, Json<User>), AppError> {
	use crate::schema::users::dsl::*;

	let mut conn = pool.get().await.map_err(AppError::internal)?;
	let user = diesel::insert_into(users)
		.values(&new_user)
		.returning(User::as_returning())
		.get_result(&mut conn)
		.await
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
pub async fn get_user(
	State(pool): State<DbPool>,
	Path(user_id): Path<Uuid>,
) -> Result<Json<User>, AppError> {
	use crate::schema::users::dsl::*;

	let mut conn = pool.get().await.map_err(AppError::internal)?;
	let user = users
		.find(user_id)
		.select(User::as_select())
		.first(&mut conn)
		.await
		.map_err(|e| match e {
			diesel::result::Error::NotFound => {
				AppError::not_found(format!("User {user_id} not found"))
			}
			other => AppError::internal(other),
		})?;

	Ok(Json(user))
}
