table! {
    user (id) {
        id -> Uuid,
        name -> Text,
        email -> Text,
        password -> Text,
        username -> Text,
        data_version -> Int4,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}
