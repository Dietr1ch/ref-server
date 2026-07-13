use serde::Serialize;
use uuid::Uuid;

/// Response body returned after a user is created.
#[derive(Debug, Clone, Serialize)]
pub struct Created {
	pub id: Uuid,
}
