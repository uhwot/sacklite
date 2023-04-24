// @generated automatically by Diesel CLI.

diesel::table! {
    user (id) {
        id -> Text,
        online_id -> Text,
        psn_id -> Nullable<BigInt>,
        rpcn_id -> Nullable<BigInt>,
        created_at -> Timestamp,
    }
}
