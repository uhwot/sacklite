use axum::{Router, routing::{get, post}, extract::{Path, State}, http::StatusCode, response::{IntoResponse, Response}, Extension};
use axum_extra::extract::Query;
use futures::TryStreamExt;
use maud::html as xml;
use serde::Deserialize;

use crate::{extractors::Xml, types::SessionData, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/userComments/:online_id", get(user_comments))
        .route("/postUserComment/:online_id", post(post_user_comment))
        .route("/deleteUserComment/:online_id", post(delete_user_comment))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommentListQuery {
    page_start: i64,
    page_size: i64,
}

async fn user_comments(
    Path(online_id): Path<String>,
    query: Query<CommentListQuery>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, Response> {
    // what the fuck have i done
    let mut comments = sqlx::query!(
        "SELECT comm.id, comm.posted_at, comm.content, comm.deleted_by_mod,
        author.online_id AS author_oid,
        deleter.online_id AS \"deleter_oid?\"
        FROM comments comm
        JOIN users author ON comm.author = author.id
        JOIN users target_user ON comm.target_user = target_user.id
        LEFT JOIN users AS deleter ON comm.deleted_by = deleter.id
        WHERE target_user.online_id = $1
        ORDER BY comm.posted_at DESC
        LIMIT $2 OFFSET $3",
        online_id,
        query.page_size,
        query.page_start - 1
    )
    .fetch(&state.pool);

    Ok(Xml(xml!(
        comments {
            @while let Some(comment) = comments.try_next().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())? {
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
    )))
}

#[derive(Deserialize)]
struct PostCommentPayload {
    message: String,
}

async fn post_user_comment(
    Path(online_id): Path<String>,
    State(state): State<AppState>,
    session: Extension<SessionData>,
    payload: Xml<PostCommentPayload>,
) -> Result<impl IntoResponse, Response> {
    let user_id = sqlx::query!("SELECT id FROM users WHERE online_id = $1", online_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?
        .ok_or((StatusCode::NOT_FOUND, "User not found").into_response())?
        .id;

    sqlx::query!(
        "INSERT INTO comments (author, target_user, content) VALUES ($1, $2, $3)",
        session.user_id,
        user_id,
        payload.message
    )
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommentDeleteQuery {
    comment_id: i64,
}

async fn delete_user_comment(
    query: Query<CommentDeleteQuery>,
    State(state): State<AppState>,
    session: Extension<SessionData>,
) -> Result<impl IntoResponse, Response> {
    let rows = sqlx::query!(
        "UPDATE comments SET deleted_by = $1
        WHERE id = $2 AND deleted_by IS NULL AND deleted_by_mod = false AND (author = $1 OR target_user = $1)",
        session.user_id,
        query.comment_id
    )
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?
    .rows_affected();

    if rows == 0 {
        return Err((StatusCode::UNAUTHORIZED, "Comment not found, already deleted or not authorized to delete").into_response())
    }

    Ok(StatusCode::OK)
}
