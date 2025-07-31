// @generated automatically by Diesel CLI.

diesel::table! {
    file_contents (id) {
        id -> Nullable<Integer>,
        size -> Integer,
        hash -> Text,
        uploader -> Text,
        created -> Text,
    }
}

diesel::table! {
    file_tags (id) {
        id -> Nullable<Integer>,
        file_id -> Integer,
        tag_id -> Integer,
    }
}

diesel::table! {
    files (id) {
        id -> Nullable<Integer>,
        name -> Text,
        content_id -> Integer,
        uploader -> Text,
        created -> Text,
    }
}

diesel::table! {
    tags (id) {
        id -> Nullable<Integer>,
        name -> Text,
        created -> Text,
    }
}

diesel::joinable!(file_tags -> files (file_id));
diesel::joinable!(file_tags -> tags (tag_id));
diesel::joinable!(files -> file_contents (content_id));

diesel::allow_tables_to_appear_in_same_query!(
    file_contents,
    file_tags,
    files,
    tags,
);
