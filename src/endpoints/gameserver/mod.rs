use axum::{Router, http::StatusCode, routing::{get, post}, middleware::{from_fn_with_state, from_fn}};
use serde::Deserialize;
use tower_http::{services::ServeFile, compression::{CompressionLayer, predicate::SizeAbove, Predicate}};

use crate::{AppState, middleware, types::Config, utils::predicate::ContentType};

mod auth;
mod comment;
mod enter_level;
mod message;
mod publish;
mod resource;
mod tags;
mod user;
mod slot_search;
mod slot;

pub async fn routes(config: &Config) -> Router<AppState> {
    let mut router = Router::new()
        // auth + digest
        .merge(enter_level::routes())
        .merge(tags::routes())
        .merge(user::routes())
        .merge(resource::routes(config.resource_size_limit))
        .merge(comment::routes())
        .merge(slot::routes())
        .merge(slot_search::routes())
        .merge(publish::routes());

    if !config.digest_key.is_empty() && config.verify_client_digest {
        router = router.layer(from_fn_with_state(config.clone(), middleware::verify_digest))
    }

    router = router
        // auth
        .merge(message::routes())
        .route_service("/network_settings.nws", ServeFile::new("network_settings.nws"))
        .route("/goodbye", post(auth::goodbye))
        .layer(from_fn(middleware::parse_session))
        // public
        .route("/login", post(auth::login))
        .route("/status", get(status));

    if !config.digest_key.is_empty() {
        router = router.layer(from_fn_with_state(config.digest_key.clone(), middleware::send_digest))
    }

    let predicate = SizeAbove::new(1024)
        .and(ContentType::const_new(mime::TEXT_XML.as_ref()));

    router = router
        .layer(
            CompressionLayer::new()
                .compress_when(predicate)
        );

    router
}

async fn status() -> StatusCode {
    StatusCode::OK
}

#[derive(Deserialize, Debug)]
struct Location {
    x: u16,
    y: u16,
}
