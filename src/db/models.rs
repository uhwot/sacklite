use std::time::SystemTime;

use bigdecimal::BigDecimal;
use diesel::prelude::*;
use uuid::Uuid;

use super::schema::users;

#[derive(Queryable)]
pub struct User {
    pub id: Uuid,
    pub online_id: String,
    pub psn_id: Option<BigDecimal>,
    pub rpcn_id: Option<BigDecimal>,
    pub created_at: SystemTime,
    pub biography: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: Uuid,
    pub online_id: String,
}
