use diesel::prelude::*;
use uuid::Uuid;

use crate::{db::models::{Comment, NewComment}};
use super::DbError;

pub fn get_comment_by_id(
    conn: &mut PgConnection,
    cid: i64,
) -> Result<Option<Comment>, DbError> {
    use crate::db::schema::comments::dsl::*;

    Ok(comments
        .filter(id.eq(cid))
        .first::<Comment>(conn)
        .optional()?)
}

pub fn get_user_comments(
    conn: &mut PgConnection,
    uid: Uuid,
    offset: i64,
    size: i64,
) -> Result<Vec<Comment>, DbError> {
    use crate::db::schema::comments::dsl::*;

    Ok(comments
        .filter(target_user.eq(uid))
        .order(posted_at.desc())
        .limit(size)
        .offset(offset)
        .load(conn)?)
}

pub fn get_user_comment_count(
    conn: &mut PgConnection,
    uid: Uuid,
) -> Result<i64, DbError> {
    use crate::db::schema::comments::dsl::*;

    Ok(comments
        .filter(target_user.eq(uid))
        .count()
        .get_result(conn)?)
}

pub fn insert_user_comment(
    conn: &mut PgConnection,
    author_id: Uuid,
    user_id: Uuid,
    content_str: &str
) -> Result<i64, DbError> {
    use crate::db::schema::comments::dsl::*;

    let comment_id = diesel::insert_into(comments)
        .values(NewComment {
            author: author_id,
            target_user: Some(user_id),
            content: content_str,
        })
        .returning(id)
        .get_result(conn)?;

    Ok(comment_id)
}

pub fn delete_comment(
    conn: &mut PgConnection,
    cid: i64,
    uid: Uuid,
) -> Result<(), DbError> {
    use crate::db::schema::comments::dsl::*;

    diesel::update(comments.filter(id.eq(cid)))
        .set(deleted_by.eq(uid))
        .execute(conn)?;

    Ok(())
}