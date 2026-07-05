use axum::{
	Json,
	extract::{Path, State},
	http::StatusCode,
};
use diesel::prelude::*;
use uuid::Uuid;

use crate::DbPool;
use crate::error::AppError;
use crate::models::user::{NewUser, User};

/// GET /users — list all users.
pub async fn list_users(State(pool): State<DbPool>) -> Result<Json<Vec<User>>, AppError> {
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
pub async fn create_user(
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
pub async fn get_user(
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
