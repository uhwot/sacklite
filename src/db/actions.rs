use diesel::prelude::*;
use uuid::Uuid;

use super::{models, wrap_to_i64};

pub type DbError = Box<dyn std::error::Error + Send + Sync>;

pub fn get_user_by_uuid(
    conn: &mut SqliteConnection,
    uuid: Uuid,
) -> Result<Option<models::User>, DbError> {
    use super::schema::user::dsl::*;

    Ok(user
        .filter(id.eq(uuid.to_string()))
        .first::<models::User>(conn)
        .optional()?)
}

pub fn get_user_by_online_id(
    conn: &mut SqliteConnection,
    oid: &str,
) -> Result<Option<models::User>, DbError> {
    use super::schema::user::dsl::*;

    Ok(user
        .filter(online_id.eq(oid))
        .first::<models::User>(conn)
        .optional()?)
}

pub fn insert_new_user(conn: &mut SqliteConnection, oid: &str) -> Result<Uuid, DbError> {
    use super::schema::user::dsl::*;

    let uuid = Uuid::new_v4();

    diesel::insert_into(user)
        .values(models::NewUser {
            id: uuid.to_string(),
            online_id: oid.to_owned(),
        })
        .execute(conn)?;

    Ok(uuid)
}

pub fn set_user_psn_id(
    conn: &mut SqliteConnection,
    uid: Uuid,
    linked_id: Option<u64>,
) -> Result<(), DbError> {
    use super::schema::user::dsl::*;

    let linked_id = linked_id.map(wrap_to_i64);

    diesel::update(user.filter(id.eq(uid.to_string())))
        .set(psn_id.eq(linked_id))
        .execute(conn)?;

    Ok(())
}

pub fn set_user_rpcn_id(
    conn: &mut SqliteConnection,
    uid: Uuid,
    linked_id: Option<u64>,
) -> Result<(), DbError> {
    use super::schema::user::dsl::*;

    let linked_id = linked_id.map(wrap_to_i64);

    diesel::update(user.filter(id.eq(uid.to_string())))
        .set(rpcn_id.eq(linked_id))
        .execute(conn)?;

    Ok(())
}
