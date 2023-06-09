use std::time::UNIX_EPOCH;

use actix_web::{web::{self, Data, ReqData, Path}, Result, Responder, error, HttpResponse};
use anyhow::Context;
use maud::html as xml;
use serde::Deserialize;

use crate::{DbPool, db::{actions::{DbError, comment::*, user::{get_user_by_online_id, get_user_by_uuid}}}, responder::Xml, types::SessionData};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentListQuery {
    page_start: u64,
    page_size: u64,
}

pub async fn user_comments(path: Path<String>, query: web::Query<CommentListQuery>, pool: Data<DbPool>) -> Result<impl Responder> {
    let online_id = path.into_inner();
    
    let resp = web::block(move || {
        let mut conn = pool.get().unwrap();

        let user = get_user_by_online_id(&mut conn, &online_id)?
            .context("User not found")?;

        let comments = get_user_comments(&mut conn, user.id, query.page_start as i64 - 1, query.page_size as i64)?;

        // what the fuck have i done
        Ok::<String, DbError>(xml!(
            comments {
                @for comment in comments {
                    comment {
                        @let author = get_user_by_uuid(&mut conn, comment.author)?.context("User not found")?;
                        id { (comment.id) }
                        npHandle { (author.online_id) }
                        timestamp { (comment.posted_at.duration_since(UNIX_EPOCH)?.as_millis()) }
                        @if comment.deleted_by_mod {
                            deleted { "true" }
                            deletedBy { "moderator" }
                            deleteType { "moderator" }
                        } @else if let Some(deleted_by) = comment.deleted_by {
                            deleted { "true" }
                            deletedBy {
                                @if deleted_by == author.id {
                                    (author.online_id)
                                } @else if deleted_by == user.id {
                                    (user.online_id)
                                }
                            }
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
        ).into_string())
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    Ok(Xml(resp))
}

#[derive(Deserialize)]
pub struct PostCommentPayload {
    message: String,
}

pub async fn post_user_comment(path: Path<String>, payload: actix_xml::Xml<PostCommentPayload>, pool: Data<DbPool>, session: ReqData<SessionData>) -> Result<impl Responder> {
    let online_id = path.into_inner();
    
    web::block(move || {
        let mut conn = pool.get().unwrap();
        
        let user = get_user_by_online_id(&mut conn, &online_id)?
            .context("User not found")?;

        insert_user_comment(&mut conn, session.user_id, user.id, &payload.message)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentDeleteQuery {
    comment_id: u64,
}

pub async fn delete_user_comment(query: web::Query<CommentDeleteQuery>, pool: Data<DbPool>, session: ReqData<SessionData>) -> Result<impl Responder> {
    let comment_id = query.comment_id as i64;
    
    let authorized = web::block(move || {
        let mut conn = pool.get().unwrap();

        let comment = get_comment_by_id(&mut conn, comment_id)?
            .context("Comment not found")?;

        if session.user_id != comment.author && comment.target_user != Some(session.user_id) {
            return Ok(false);
        }

        delete_comment(&mut conn, comment_id, session.user_id)?;

        Ok::<bool, DbError>(true)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    match authorized {
        true => Ok(HttpResponse::Ok()),
        false => Err(error::ErrorUnauthorized("")),
    }
}