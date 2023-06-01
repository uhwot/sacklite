// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Uuid,
        online_id -> Varchar,
        psn_id -> Nullable<Numeric>,
        rpcn_id -> Nullable<Numeric>,
        created_at -> Timestamp,
        biography -> Varchar,
    }
}
