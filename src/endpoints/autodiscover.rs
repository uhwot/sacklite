use crate::types::Config;
use actix_web::{
    web::{Data, Json},
    Responder,
};
use serde_json::json;

// docs:
// https://github.com/LittleBigRefresh/Docs/blob/c770a444949b3902c7b16e57f840da10bb279159/autodiscover-api.md

pub async fn autodiscover(config: Data<Config>) -> impl Responder {
    Json(json!({
        "version": 2,
        "serverBrand": "sacklite",
        "url": config.external_url.clone() + &config.base_path,
        "usesCustomDigestKey": config.digest_key == "CustomServerDigest",
    }))
}
