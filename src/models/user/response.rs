use serde::Serialize;
use uuid::Uuid;

/// Response body returned after a user is created.
#[derive(Debug, Clone, Serialize)]
pub struct Created {
	pub id: Uuid,
}

/// Response body returned when `Prefer: return=representation` is set on PATCH.
///
/// Excludes internal fields (~created_at~) that clients should not see.
#[derive(Debug, Clone, Serialize)]
pub struct Patched {
	pub id: Uuid,
	pub name: String,
	pub email: String,
}

impl From<&crate::models::user::User> for Patched {
	fn from(user: &crate::models::user::User) -> Self {
		Self {
			id: user.id,
			name: user.name.clone(),
			email: user.email.clone(),
		}
	}
}
