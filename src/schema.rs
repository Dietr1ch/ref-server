// @generated automatically by Diesel CLI.

diesel::table! {
	users (id) {
		id -> Uuid,
		#[max_length = 255]
		name -> Varchar,
		#[max_length = 255]
		email -> Varchar,
		created_at -> Timestamptz,
	}
}

diesel::table! {
	posts (id) {
		id -> Uuid,
		user_id -> Uuid,
		#[max_length = 255]
		title -> Varchar,
		body -> Nullable<Text>,
		created_at -> Timestamptz,
	}
}

diesel::joinable!(posts -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(posts, users,);
