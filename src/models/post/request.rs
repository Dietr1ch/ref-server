use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPayload {
	pub title: String,
	pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::posts)]
pub struct New {
	pub user_id: Uuid,
	pub title: String,
	pub body: Option<String>,
}
