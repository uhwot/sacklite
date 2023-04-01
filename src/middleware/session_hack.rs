use actix_web::{
    body::{MessageBody, BodySize},
    dev::{self, ServiceResponse},
    web::Bytes,
    Error,
};
use actix_web_lab::middleware::Next;
use actix_http::header::SET_COOKIE;

pub async fn session_hack(req: dev::ServiceRequest, next: Next<impl MessageBody + 'static>) -> Result<ServiceResponse<impl MessageBody>, Error> {
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