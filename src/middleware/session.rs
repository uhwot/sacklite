use std::str::FromStr;

use actix_http::{
    header::SET_COOKIE,
    StatusCode, HttpMessage,
};
use actix_session::SessionExt;
use actix_web::{
    body::{BodySize, EitherBody, MessageBody},
    dev::{self, ServiceResponse},
    web::Bytes,
    Error, HttpResponse,
};
use actix_web_lab::middleware::Next;
use uuid::Uuid;

use crate::types::{SessionData, Platform, GameVersion};

pub async fn session_hack(
    req: dev::ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let path = req.path().to_owned();
    let mut res = next.call(req).await?;

    if !path.ends_with("/login") {
        res.headers_mut().remove(SET_COOKIE);
        return Ok(res.map_into_boxed_body());
    }

    let auth_ticket = match res.headers().get(SET_COOKIE) {
        Some(c) => c.clone(),
        None => return Ok(res.map_into_boxed_body()),
    };
    let auth_ticket = auth_ticket.to_str().unwrap().split_once(';').unwrap().0;

    res.headers_mut().remove(SET_COOKIE);

    // code still stolen from here:
    // https://github.com/chriswk/actix-middleware-etag/blob/fe10145fa730d9c45deb7e05c594ad5760b9761a/src/lib.rs#L103
    let res = res.map_body(|_h, body| match body.size() {
        BodySize::Sized(_size) => {
            let body = body.try_into_bytes().unwrap_or_else(|_| Bytes::new());
            let body = std::str::from_utf8(&body).unwrap();
            let body = body.replacen("ass", auth_ticket, 1);
            Bytes::copy_from_slice(body.as_bytes()).boxed()
        }
        _ => body.boxed(),
    });

    Ok(res)
}

pub async fn parse_session(
    req: dev::ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<EitherBody<impl MessageBody>>, Error> {
    if req.path().ends_with("/login") {
        let res = next.call(req).await?;
        return Ok(res.map_into_left_body())
    }

    let session = req.get_session();
    
    let user_id: Option<String> = session.get("user_id").unwrap();
    if let None = user_id {
        let (req, _pl) = req.into_parts();
        let res = HttpResponse::new(StatusCode::FORBIDDEN).map_into_right_body();
        return Ok(ServiceResponse::new(req, res));
    }

    // TODO: check if user exists in database

    let platform: String = session.get("platform").unwrap().unwrap();
    let game_version: String = session.get("game_version").unwrap().unwrap();

    let session_data = SessionData {
        user_id: Uuid::parse_str(&user_id.unwrap()).unwrap(),
        online_id: session.get("online_id").unwrap().unwrap(),
        platform: Platform::from_str(&platform).unwrap(),
        game_version: GameVersion::from_str(&game_version).unwrap(),
    };

    req.extensions_mut().insert(session_data);

    let res = next.call(req).await?;

    Ok(res.map_into_left_body())
}