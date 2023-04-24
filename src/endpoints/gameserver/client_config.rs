use actix_web::{error, HttpResponse, Responder, Result};
use std::fs;

pub async fn network_settings() -> Result<impl Responder> {
    let file_bytes = fs::read("network_settings.nws").map_err(|_| error::ErrorNotFound(""))?;

    Ok(HttpResponse::Ok()
        // skips compression since LBP is stupid and sends "accept-encoding" header anyways
        .insert_header(("content-encoding", "identity"))
        .body(file_bytes))
}
