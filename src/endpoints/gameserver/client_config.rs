use actix_web::{error, HttpResponse, Responder, Result};
use std::fs;

pub async fn network_settings() -> Result<impl Responder> {
    let file_bytes = fs::read("network_settings.nws").map_err(|_| error::ErrorNotFound(""))?;

    // skips compression since LBP is stupid
    // and sends "accept-encoding" header anyways even though it doesn't work
    Ok(HttpResponse::Ok()
        .insert_header(("content-encoding", "identity"))
        .body(file_bytes))
}
