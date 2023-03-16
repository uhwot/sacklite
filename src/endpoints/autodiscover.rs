use actix_web::{web, Responder};
use serde::Serialize;
use crate::types::config::Config;

// docs:
// https://github.com/LittleBigRefresh/Docs/blob/main/autodiscover-api.md

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Autodiscover {
    version: u32,
    server_brand: &'static str,
    url: String,
}

pub async fn autodiscover(data: web::Data<Config>) -> impl Responder {
    web::Json(Autodiscover {
        version: 1,
        server_brand: "sacklite",
        url: data.autodiscover_url.clone(),
    })
}