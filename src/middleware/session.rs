use axum::{extract::Request, middleware::Next, response::{Response, IntoResponse}, http::StatusCode};
use tower_sessions::Session;
use uuid::Uuid;

use crate::types::SessionData;

pub async fn remove_set_cookie(
    req: Request,
    next: Next,
) -> Response {
    let mut resp = next.run(req).await;
    resp.headers_mut().remove("Set-Cookie");
    resp
}

pub async fn parse_session(
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, Response> {
    let session: &Session = req.extensions().get().ok_or_else(|| StatusCode::FORBIDDEN.into_response())?;

    let user_id: Option<String> = session.get("user_id").await.unwrap();
    if user_id.is_none() {
        return Err(StatusCode::FORBIDDEN.into_response());
    }

    let platform: u8 = session.get("platform").await.unwrap().unwrap();
    let game_version: u8 = session.get("game_version").await.unwrap().unwrap();

    let session_data = SessionData {
        user_id: Uuid::parse_str(&user_id.unwrap()).unwrap(),
        online_id: session.get("online_id").await.unwrap().unwrap(),
        platform: platform.try_into().unwrap(),
        game_version: game_version.try_into().unwrap(),
    };

    req.extensions_mut().insert(session_data);

    Ok(next.run(req).await)
}
