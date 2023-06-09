use bigdecimal::BigDecimal;
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::models::{User, NewUser};
use super::DbError;

pub fn get_user_by_uuid(
    conn: &mut PgConnection,
    uuid: Uuid,
) -> Result<Option<User>, DbError> {
    use crate::db::schema::users::dsl::*;

    Ok(users
        .filter(id.eq(uuid))
        .first::<User>(conn)
        .optional()?)
}

pub fn get_user_by_online_id(
    conn: &mut PgConnection,
    oid: &str,
) -> Result<Option<User>, DbError> {
    use crate::db::schema::users::dsl::*;

    Ok(users
        .filter(online_id.eq(oid))
        .first::<User>(conn)
        .optional()?)
}

pub fn get_user_by_psn_id(
    conn: &mut PgConnection,
    linked_id: u64,
) -> Result<Option<User>, DbError> {
    use crate::db::schema::users::dsl::*;

    let linked_id = BigDecimal::from(linked_id);

    Ok(users
        .filter(psn_id.eq(linked_id))
        .first::<User>(conn)
        .optional()?)
}

pub fn get_user_by_rpcn_id(
    conn: &mut PgConnection,
    linked_id: u64,
) -> Result<Option<User>, DbError> {
    use crate::db::schema::users::dsl::*;

    let linked_id = BigDecimal::from(linked_id);

    Ok(users
        .filter(rpcn_id.eq(linked_id))
        .first::<User>(conn)
        .optional()?)
}

pub fn insert_new_user(conn: &mut PgConnection, oid: &str) -> Result<Uuid, DbError> {
    use crate::db::schema::users::dsl::*;

    let uuid = Uuid::new_v4();

    diesel::insert_into(users)
        .values(NewUser {
            id: uuid,
            online_id: oid.to_owned(),
        })
        .execute(conn)?;

    Ok(uuid)
}

pub fn set_user_online_id(
    conn: &mut PgConnection,
    uid: Uuid,
    oid: &str,
) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(uid)))
        .set(online_id.eq(oid))
        .execute(conn)?;

    Ok(())
}

pub fn set_user_psn_id(
    conn: &mut PgConnection,
    uid: Uuid,
    linked_id: Option<u64>,
) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;

    let linked_id = linked_id.map(BigDecimal::from);

    diesel::update(users.filter(id.eq(uid)))
        .set(psn_id.eq(linked_id))
        .execute(conn)?;

    Ok(())
}

pub fn set_user_rpcn_id(
    conn: &mut PgConnection,
    uid: Uuid,
    linked_id: Option<u64>,
) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;

    let linked_id = linked_id.map(BigDecimal::from);

    diesel::update(users.filter(id.eq(uid)))
        .set(rpcn_id.eq(linked_id))
        .execute(conn)?;

    Ok(())
}

pub fn set_user_location(
    conn: &mut PgConnection,
    uid: Uuid,
    x: u16,
    y: u16,
) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(uid)))
        .set((location_x.eq(x as i32), location_y.eq(y as i32)))
        .execute(conn)?;

    Ok(())
}

pub fn set_user_biography(
    conn: &mut PgConnection,
    uid: Uuid,
    bio: &str,
) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(uid)))
        .set(biography.eq(bio))
        .execute(conn)?;

    Ok(())
}

pub fn set_user_icon(
    conn: &mut PgConnection,
    uid: Uuid,
    icon_ref: &str,
) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(uid)))
        .set(icon.eq(icon_ref))
        .execute(conn)?;

    Ok(())
}

pub fn set_user_lbp2_planets(
    conn: &mut PgConnection,
    uid: Uuid,
    planets_ref: &str,
) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(uid)))
        .set(lbp2_planets.eq(planets_ref))
        .execute(conn)?;

    Ok(())
}

pub fn set_user_lbp3_planets(
    conn: &mut PgConnection,
    uid: Uuid,
    planets_ref: &str,
) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(uid)))
        .set(lbp3_planets.eq(planets_ref))
        .execute(conn)?;

    Ok(())
}

pub fn set_user_ccplanet(
    conn: &mut PgConnection,
    uid: Uuid,
    ccplanet_ref: &str,
) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(uid)))
        .set(cross_control_planet.eq(ccplanet_ref))
        .execute(conn)?;

    Ok(())
}

pub fn set_user_yay2(
    conn: &mut PgConnection,
    uid: Uuid,
    yay2_ref: &str,
) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(uid)))
        .set(yay2.eq(yay2_ref))
        .execute(conn)?;

    Ok(())
}

pub fn set_user_meh2(
    conn: &mut PgConnection,
    uid: Uuid,
    meh2_ref: &str,
) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(uid)))
        .set(meh2.eq(meh2_ref))
        .execute(conn)?;

    Ok(())
}

pub fn set_user_boo2(
    conn: &mut PgConnection,
    uid: Uuid,
    boo2_ref: &str,
) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(uid)))
        .set(boo2.eq(boo2_ref))
        .execute(conn)?;

    Ok(())
}

pub fn set_user_awards(
    conn: &mut PgConnection,
    uid: Uuid,
    awards_vec: &Vec<i64>,
) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(uid)))
        .set(awards.eq(awards_vec))
        .execute(conn)?;

    Ok(())
}

pub fn set_user_progress(
    conn: &mut PgConnection,
    uid: Uuid,
    progress_vec: &Vec<i64>,
) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(uid)))
        .set(progress.eq(progress_vec))
        .execute(conn)?;

    Ok(())
}

pub fn set_user_profile_pins(
    conn: &mut PgConnection,
    uid: Uuid,
    pins_vec: &Vec<i64>,
) -> Result<(), DbError> {
    use crate::db::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(uid)))
        .set(profile_pins.eq(pins_vec))
        .execute(conn)?;

    Ok(())
}