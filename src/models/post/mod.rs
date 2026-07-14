pub mod request;
pub mod response;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Post {
	pub id: Uuid,
	pub user_id: Uuid,
	pub title: String,
	pub body: String,
	pub created_at: DateTime<Utc>,
}
