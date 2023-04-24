use diesel::{prelude::*};
use time::PrimitiveDateTime;

use super::schema::user;

#[derive(Queryable)]
pub struct User {
    pub id: String,
    pub online_id: String,
    // linked ids are wrapped from u64 nums
    pub psn_id: Option<i64>,
    pub rpcn_id: Option<i64>,
    pub created_at: PrimitiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = user)]
pub struct NewUser {
    pub id: String,
    pub online_id: String,
}
