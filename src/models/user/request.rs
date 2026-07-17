use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Input payload for creating a new user.
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct New {
	pub name: String,
	pub email: String,
}

/// Input payload for partially updating a user (RFC 7386 JSON Merge Patch).
///
/// Only the fields present in the request body are updated.
#[derive(Debug, Clone, Deserialize)]
pub struct Patch {
	pub name: Option<String>,
}
