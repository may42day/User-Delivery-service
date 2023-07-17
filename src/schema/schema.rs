// @generated automatically by Diesel CLI.

diesel::table! {
    couriers (user_uuid) {
        user_uuid -> Uuid,
        is_free -> Bool,
        rating -> Float8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (uuid) {
        uuid -> Uuid,
        first_name -> Text,
        address -> Nullable<Text>,
        phone_number -> Text,
        email -> Text,
        password -> Text,
        role -> Text,
        is_blocked -> Bool,
        is_deleted -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users_queue (id) {
        id -> Int8,
        user_uuid -> Uuid,
        status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(couriers -> users (user_uuid));

diesel::allow_tables_to_appear_in_same_query!(
    couriers,
    users,
    users_queue,
);
