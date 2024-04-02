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
        exif_date -> Integer,
        exif_json -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(duplicates, photos,);
