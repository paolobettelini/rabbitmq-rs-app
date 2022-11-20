// @generated automatically by Diesel CLI.

diesel::table! {
    image (id, user_id) {
        id -> Integer,
        user_id -> Integer,
        uploaded_at -> Timestamp,
        data -> Blob,
    }
}

diesel::table! {
    user (id) {
        id -> Integer,
        mail -> Varchar,
        username -> Varchar,
        password -> Binary,
        token -> Binary,
        created_at -> Timestamp,
    }
}

diesel::joinable!(image -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    image,
    user,
);
