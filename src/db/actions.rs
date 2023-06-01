use bigdecimal::BigDecimal;
use diesel::prelude::*;
use uuid::Uuid;

use super::models;

pub type DbError = Box<dyn std::error::Error + Send + Sync>;

pub fn get_user_by_uuid(
    conn: &mut PgConnection,
    uuid: Uuid,
) -> Result<Option<models::User>, DbError> {
    use super::schema::users::dsl::*;

    Ok(users
        .filter(id.eq(uuid))
        .first::<models::User>(conn)
        .optional()?)
}

pub fn get_user_by_online_id(
    conn: &mut PgConnection,
    oid: &str,
) -> Result<Option<models::User>, DbError> {
    use super::schema::users::dsl::*;

    Ok(users
        .filter(online_id.eq(oid))
        .first::<models::User>(conn)
        .optional()?)
}

pub fn get_user_by_psn_id(
    conn: &mut PgConnection,
    linked_id: u64,
) -> Result<Option<models::User>, DbError> {
    use super::schema::users::dsl::*;

    let linked_id = BigDecimal::from(linked_id);

    Ok(users
        .filter(psn_id.eq(linked_id))
        .first::<models::User>(conn)
        .optional()?)
}

pub fn get_user_by_rpcn_id(
    conn: &mut PgConnection,
    linked_id: u64,
) -> Result<Option<models::User>, DbError> {
    use super::schema::users::dsl::*;

    let linked_id = BigDecimal::from(linked_id);

    Ok(users
        .filter(rpcn_id.eq(linked_id))
        .first::<models::User>(conn)
        .optional()?)
}

pub fn insert_new_user(conn: &mut PgConnection, oid: &str) -> Result<Uuid, DbError> {
    use super::schema::users::dsl::*;

    let uuid = Uuid::new_v4();

    diesel::insert_into(users)
        .values(models::NewUser {
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
    use super::schema::users::dsl::*;

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
    use super::schema::users::dsl::*;

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
    use super::schema::users::dsl::*;

    let linked_id = linked_id.map(BigDecimal::from);

    diesel::update(users.filter(id.eq(uid)))
        .set(rpcn_id.eq(linked_id))
        .execute(conn)?;

    Ok(())
}
