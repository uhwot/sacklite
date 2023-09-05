use std::sync::Arc;

use actix_web::{
    error,
    web::{Data, Path, ReqData, Query},
    HttpResponse, Responder, Result,
};
use futures::TryStreamExt;
use maud::html as xml;
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::{responder::Xml, types::SessionData};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentListQuery {
    page_start: u64,
    page_size: u64,
}

pub async fn user_comments(
    path: Path<String>,
    query: Query<CommentListQuery>,
    pool: Data<Arc<Pool<Postgres>>>,
) -> Result<impl Responder> {
    let online_id = path.into_inner();

    let user_id = sqlx::query!("SELECT id FROM users WHERE online_id = $1", online_id)
        .fetch_optional(&***pool)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or(error::ErrorNotFound("User not found"))?
        .id;

    // what the fuck have i done
    let mut comments = sqlx::query!(
        "SELECT comm.id, comm.posted_at, comm.content, comm.deleted_by_mod,
        author.online_id AS author_oid,
        deleter.online_id AS \"deleter_oid?\"
        FROM comments comm
        JOIN users author ON comm.author = author.id
        LEFT JOIN users AS deleter ON comm.deleted_by = deleter.id
        WHERE comm.target_user = $1
        ORDER BY comm.posted_at DESC
        LIMIT $2 OFFSET $3",
        user_id,
        query.page_size as i64,
        query.page_start as i64 - 1
    )
    .fetch(&***pool);

    Ok(Xml(xml!(
        comments {
            @while let Some(comment) = comments.try_next().await.map_err(error::ErrorInternalServerError)? {
                comment {
                    id { (comment.id) }
                    npHandle { (comment.author_oid) }
                    timestamp { (comment.posted_at.timestamp_millis()) }
                    @if comment.deleted_by_mod {
                        deleted { "true" }
                        deletedBy { "moderator" }
                        deleteType { "moderator" }
                    } @else if let Some(deleter_oid) = comment.deleter_oid {
                        deleted { "true" }
                        deletedBy { (deleter_oid) }
                        deleteType { "user" }
                    } @else {
                        message { (comment.content) }
                    }
                    thumbsup { "0" } // TODO: fix this once ratings are implemented
                    thumbsdown { "0" }
                    yourthumb { "0" }
                }
            }
        }
    ).into_string()))
}

#[derive(Deserialize)]
pub struct PostCommentPayload {
    message: String,
}

pub async fn post_user_comment(
    path: Path<String>,
    payload: actix_xml::Xml<PostCommentPayload>,
    pool: Data<Arc<Pool<Postgres>>>,
    session: ReqData<SessionData>,
) -> Result<impl Responder> {
    let online_id = path.into_inner();

    let user_id = sqlx::query!("SELECT id FROM users WHERE online_id = $1", online_id)
        .fetch_optional(&***pool)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or(error::ErrorNotFound("User not found"))?
        .id;

    sqlx::query!(
        "INSERT INTO comments (author, target_user, content) VALUES ($1, $2, $3)",
        session.user_id,
        user_id,
        payload.message
    )
    .execute(&***pool)
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentDeleteQuery {
    comment_id: u64,
}

pub async fn delete_user_comment(
    query: Query<CommentDeleteQuery>,
    pool: Data<Arc<Pool<Postgres>>>,
    session: ReqData<SessionData>,
) -> Result<impl Responder> {
    let rows = sqlx::query!(
        "UPDATE comments SET deleted_by = $1
        WHERE id = $2 AND deleted_by IS NULL AND deleted_by_mod = false AND (author = $1 OR target_user = $1)",
        session.user_id,
        query.comment_id as i64
    )
    .execute(&***pool)
    .await
    .map_err(error::ErrorInternalServerError)?
    .rows_affected();

    if rows == 0 {
        return Err(error::ErrorUnauthorized("Comment not found, already deleted or not authorized to delete"))
    }

    Ok(HttpResponse::Ok())
}
