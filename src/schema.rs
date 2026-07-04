// @generated automatically by Diesel CLI, manually maintained for now.

diesel::table! {
	users (id) {
		id -> Uuid,
		name -> Varchar,
		email -> Varchar,
		created_at -> Timestamptz,
	}
}
