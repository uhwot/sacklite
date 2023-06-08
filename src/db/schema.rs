// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Uuid,
        online_id -> Varchar,
        psn_id -> Nullable<Numeric>,
        rpcn_id -> Nullable<Numeric>,
        created_at -> Timestamp,
        biography -> Varchar,
        location_x -> Int4,
        location_y -> Int4,
        icon -> Varchar,
        lbp2_planets -> Varchar,
        lbp3_planets -> Varchar,
        cross_control_planet -> Varchar,
        yay2 -> Varchar,
        meh2 -> Varchar,
        boo2 -> Varchar,
        awards -> Array<Int8>,
        progress -> Array<Int8>,
        profile_pins -> Array<Int8>,
    }
}
