use crate::AppState;
use axum::{extract::State, Json};
use serde_json::{json, Value};

// docs:
// https://docs.littlebigrefresh.com/autodiscover-api.html

pub async fn autodiscover(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "version": 3,
        "serverBrand": "sacklite",
        "url": state.config.external_url.clone() + &state.config.base_path,
        "usesCustomDigestKey": state.config.digest_key == "CustomServerDigest",
        "serverDescription": state.config.server_desc.clone(),
        "bannerImageUrl": state.config.banner_image_url,
    }))
}
