use diesel::prelude::*;

use super::schema::user;

#[derive(Queryable)]
pub struct User {
    pub id: String,
    pub online_id: String,
    pub psn_id: Option<i64>,
    pub rpcn_id: Option<i64>,
    pub created_at: i64,
}

#[derive(Insertable)]
#[diesel(table_name = user)]
pub struct NewUser {
    pub id: String,
    pub online_id: String,
}