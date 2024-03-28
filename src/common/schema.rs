// @generated automatically by Diesel CLI.

diesel::table! {
    sudos (id) {
        id -> Integer,
        username -> Text,
        hostnames -> Text,
    }
}
