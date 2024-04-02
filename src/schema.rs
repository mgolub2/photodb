// @generated automatically by Diesel CLI.

diesel::table! {
    duplicates (id) {
        id -> Integer,
        hash -> BigInt,
        original_path -> Text,
        deleted -> Bool,
    }
}

diesel::table! {
    photos (hash) {
        hash -> BigInt,
        original_path -> Text,
        current_path -> Text,
        exif_json -> Text,
        exif_date -> BigInt,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    duplicates,
    photos,
);
