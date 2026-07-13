use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Input payload for creating a new user.
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct New {
	pub name: String,
	pub email: String,
}
