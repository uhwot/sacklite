use crate::types::Config;
use actix_web::{web, Responder};
use serde::Serialize;

// docs:
// https://github.com/LittleBigRefresh/Docs/blob/c770a444949b3902c7b16e57f840da10bb279159/autodiscover-api.md

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
        url: config.external_url.clone() + &config.base_path,
        uses_custom_digest_key: config.digest_key == "CustomServerDigest",
    })
}
