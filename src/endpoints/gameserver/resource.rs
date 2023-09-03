use std::fs;

use actix_web::{
    error,
    web::{Bytes, Data, Path},
    HttpResponse, Responder, Result,
};
use log::debug;
use maud::html as xml;
use serde::Deserialize;
use serde_with::serde_as;
use sha1::{Digest, Sha1};

use crate::{
    responder::Xml,
    types::Config,
    utils::resource::{get_hash_path, str_to_hash},
};

pub async fn download(path: Path<String>, config: Data<Config>) -> Result<impl Responder> {
    let hash = path.into_inner();
    let hash = str_to_hash(&hash).map_err(|_| {
        error::ErrorBadRequest(format!("Resource SHA1 hash is invalid: {hash}"))
    })?;

    let path = get_hash_path(&config.resource_dir, hash);

    let file = fs::read(path).map_err(|e| {
        debug!("Couldn't read resource file: {e}");
        error::ErrorNotFound("")
    })?;

    Ok(HttpResponse::Ok()
        .content_type(mime::APPLICATION_OCTET_STREAM)
        .body(file))
}

pub async fn upload(
    payload: Bytes,
    path: Path<String>,
    config: Data<Config>,
) -> Result<impl Responder> {
    let hash = path.into_inner();
    let hash = str_to_hash(&hash).map_err(|_| {
        error::ErrorBadRequest(format!("Resource SHA1 hash is invalid: {hash}"))
    })?;

    let mut hasher = Sha1::new();
    hasher.update(&payload);

    if hash != hasher.finalize()[..] {
        return Err(error::ErrorBadRequest("Actual resource hash doesn't match hash in request"));
    }

    // TODO: add more checks n shit

    let path = get_hash_path(&config.resource_dir, hash);

    if path.exists() {
        return Err(error::ErrorConflict("Resource is already uploaded"));
    }

    fs::create_dir_all(&config.resource_dir).map_err(|e| {
        error::ErrorInternalServerError(format!("Couldn't create resource dir: {e}"))
    })?;
    fs::write(path, &payload).map_err(|e| {
        error::ErrorInternalServerError(format!("Couldn't write to resource file: {e}"))
    })?;

    Ok(HttpResponse::Ok())
}

#[serde_as]
#[derive(Deserialize)]
pub struct ResourceList {
    #[serde(default)]
    #[serde_as(as = "Vec<serde_with::hex::Hex>")]
    resource: Vec<[u8; 20]>,
}

pub async fn filter_resources(
    payload: actix_xml::Xml<ResourceList>,
    config: Data<Config>,
) -> Result<impl Responder> {
    {
        Ok(Xml(xml!(
            resources {
                @for hash in &payload.resource {
                    @if !get_hash_path(&config.resource_dir, *hash).exists() {
                        resource { (hex::encode(hash)) }
                    }
                }
            }
        )
        .into_string()))
    }
}
