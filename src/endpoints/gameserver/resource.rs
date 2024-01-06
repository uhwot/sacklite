use std::fs;

use axum::{
    Router,
    routing::{get, post},
    extract::{Path, State},
    response::{IntoResponse, Response},
    http::{StatusCode, HeaderValue},
    body::{Bytes, Body}
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tower_http::limit::RequestBodyLimitLayer;
use tracing::debug;
use maud::html as xml;
use serde::Deserialize;
use serde_with::serde_as;
use sha1::{Digest, Sha1};

use crate::{utils::resource::{get_hash_path, str_to_hash}, AppState, responders::Xml, extractors};

pub fn routes(resource_size_limit: u32) -> Router<AppState> {
    Router::new()
        .route("/r/:hash", get(download)).layer(RequestBodyLimitLayer::new(resource_size_limit as usize))
        .route("/upload/:hash", post(upload))
        .route("/filterResources", get(filter_resources))
        .route("/showNotUploaded", get(filter_resources))
}

async fn download(Path(hash): Path<String>, State(state): State<AppState>) -> Result<impl IntoResponse, Response> {
    let hash = str_to_hash(&hash).map_err(|_| {
        (StatusCode::BAD_REQUEST, format!("Resource SHA1 hash is invalid: {hash}")).into_response()
    })?;

    let path = get_hash_path(&state.config.resource_dir, hash);

    let file = File::open(path).await.map_err(|e| {
        debug!("Couldn't read resource file: {e}");
        (StatusCode::NOT_FOUND, "Resource not found").into_response()
    })?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mut resp = Response::new(body);
    resp.headers_mut().insert("Content-Type", HeaderValue::from_static(mime::APPLICATION_OCTET_STREAM.as_ref()));

    Ok(resp)
}

async fn upload(
    Path(hash): Path<String>,
    State(state): State<AppState>,
    payload: Bytes,
) -> Result<impl IntoResponse, Response> {
    let hash = str_to_hash(&hash).map_err(|_| {
        (StatusCode::BAD_REQUEST, format!("Resource SHA1 hash is invalid: {hash}")).into_response()
    })?;

    let mut hasher = Sha1::new();
    hasher.update(&payload);

    if hash != hasher.finalize()[..] {
        return Err((StatusCode::BAD_REQUEST, "Actual resource hash doesn't match hash in request").into_response());
    }

    // TODO: add more checks n shit

    let path = get_hash_path(&state.config.resource_dir, hash);

    if path.exists() {
        return Err((StatusCode::CONFLICT, "Resource is already uploaded").into_response());
    }

    fs::create_dir_all(&state.config.resource_dir).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Couldn't create resource dir: {e}")).into_response()
    })?;
    fs::write(path, &payload).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Couldn't write to resource file: {e}")).into_response()
    })?;

    Ok(StatusCode::OK)
}

#[serde_as]
#[derive(Deserialize)]
struct ResourceList {
    #[serde(default)]
    #[serde_as(as = "Vec<serde_with::hex::Hex>")]
    resource: Vec<[u8; 20]>,
}

async fn filter_resources(
    State(state): State<AppState>,
    payload: extractors::Xml<ResourceList>,
) -> impl IntoResponse {
    {
        Xml(xml!(
            resources {
                @for hash in &payload.resource {
                    @if !get_hash_path(&state.config.resource_dir, *hash).exists() {
                        resource { (hex::encode(hash)) }
                    }
                }
            }
        ))
    }
}
