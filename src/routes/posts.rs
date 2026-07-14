use axum::{
	Json,
	extract::{Path, State},
	http::StatusCode,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::app;
use crate::models::post::Post;
use crate::models::post::request;
use crate::models::post::response;
use crate::schema::posts::columns as posts_cols;
use crate::schema::posts::dsl::posts as posts_table;

/// GET /users/{user_id}/posts — list all posts for a user.
pub async fn list_posts(
	State(pool): State<app::DbPool>,
	Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<Post>>, app::Error> {
	let mut conn = pool.get().await.map_err(app::Error::internal)?;

	let items = posts_table
		.filter(posts_cols::user_id.eq(user_id))
		.select(Post::as_select())
		.load(&mut conn)
		.await
		.map_err(app::Error::internal)?;

	Ok(Json(items))
}

/// POST /users/{user_id}/posts — create a new post for a user.
pub async fn create_post(
	State(pool): State<app::DbPool>,
	Path(user_id): Path<Uuid>,
	Json(payload): Json<request::NewPayload>,
) -> Result<(StatusCode, Json<response::Created>), app::Error> {
	use crate::schema::users::dsl as users;

	let mut conn = pool.get().await.map_err(app::Error::internal)?;

	// Verify the user exists before creating the post.
	let _user: crate::models::user::User = users::users
		.find(user_id)
		.select(crate::models::user::User::as_select())
		.first(&mut conn)
		.await
		.map_err(|e| match e {
			diesel::result::Error::NotFound => {
				app::Error::not_found(format!("User {user_id} not found"))
			}
			other => app::Error::internal(other),
		})?;

	let new_post = diesel::insert_into(posts_table)
		.values(&request::New {
			user_id,
			title: payload.title,
			body: payload.body.filter(|b| !b.is_empty()),
		})
		.returning(Post::as_returning())
		.get_result(&mut conn)
		.await
		.map_err(app::Error::internal)?;

	Ok((
		StatusCode::CREATED,
		Json(response::Created { id: new_post.id }),
	))
}
