use std::time::SystemTime;

use bigdecimal::BigDecimal;
use diesel::prelude::*;
use uuid::Uuid;

use super::schema::*;

#[derive(Queryable)]
pub struct User {
    pub id: Uuid,
    pub online_id: String,
    pub psn_id: Option<BigDecimal>,
    pub rpcn_id: Option<BigDecimal>,
    pub created_at: SystemTime,
    pub biography: String,
    pub location_x: i32,
    pub location_y: i32,
    pub icon: String,
    pub lbp2_planets: String,
    pub lbp3_planets: String,
    pub cross_control_planet: String,
    pub yay2: String,
    pub meh2: String,
    pub boo2: String,
    pub awards: Vec<i64>,
    pub progress: Vec<i64>,
    pub profile_pins: Vec<i64>,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: Uuid,
    pub online_id: String,
}

#[derive(Queryable)]
pub struct Comment {
    pub id: i64,
    pub author: Uuid,
    pub posted_at: SystemTime,
    pub target_user: Option<Uuid>,
    pub content: String,
    pub deleted_by: Option<Uuid>,
    pub deleted_by_mod: bool,
}

#[derive(Insertable)]
#[diesel(table_name = comments)]
pub struct NewComment<'a> {
    pub author: Uuid,
    pub target_user: Option<Uuid>,
    pub content: &'a str,
}