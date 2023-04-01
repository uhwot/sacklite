use actix_web::{
    body::MessageBody,
    dev::{self, ServiceResponse},
    Error,
};
use actix_web_lab::middleware::Next;
use actix_http::header::SET_COOKIE;
use log::debug;

pub async fn session_hack(req: dev::ServiceRequest, next: Next<impl MessageBody + 'static>) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let res = next.call(req).await?;
    debug!("{:?}", res.headers().get(SET_COOKIE));
    Ok(res)
}