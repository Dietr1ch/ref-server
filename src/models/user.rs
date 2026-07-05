use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A user as stored in the database and returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
	pub id: Uuid,
	pub name: String,
	pub email: String,
	pub created_at: DateTime<Utc>,
}

/// Input payload for creating a new user.
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
	pub name: String,
	pub email: String,
}
