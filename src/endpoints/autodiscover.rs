use crate::types::Config;
use actix_web::{web, Responder};
use serde::Serialize;

// docs:
// https://github.com/LittleBigRefresh/Docs/blob/main/autodiscover-api.md

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Autodiscover {
    version: u32,
    server_brand: &'static str,
    url: String,
    uses_custom_digest_key: bool,
}

pub async fn autodiscover(config: web::Data<Config>) -> impl Responder {
    web::Json(Autodiscover {
        version: 2,
        server_brand: "sacklite",
        url: config.autodiscover_url.clone(),
        uses_custom_digest_key: config.digest_key == "CustomServerDigest",
    })
}
