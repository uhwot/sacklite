use std::fs;

use actix_web::{error, web, HttpResponse, Responder, Result};
use log::{debug, error};
use maud::html as xml;
use serde::Deserialize;
use sha1::{Sha1, Digest};

use crate::{types::Config, utils::resource::{check_sha1, get_res_path}, responder::Xml};

pub async fn download(path: web::Path<String>, config: web::Data<Config>) -> Result<impl Responder> {
    let mut hash = path.into_inner();
    hash.make_ascii_lowercase();

    if !check_sha1(&hash) {
        debug!("Resource SHA1 hash is invalid: {hash}");
        return Err(error::ErrorBadRequest(""))
    };

    let path = get_res_path(&config.resource_dir, &hash);

    let file = fs::read(path).map_err(|e| {
        debug!("Couldn't read resource file: {e}");
        error::ErrorNotFound("")
    })?;

    Ok(
        HttpResponse::Ok()
            .content_type(mime::APPLICATION_OCTET_STREAM)
            .body(file)
    )
}

pub async fn upload(payload: web::Bytes, path: web::Path<String>, config: web::Data<Config>) -> Result<impl Responder> {
    let hash = path.into_inner();

    let mut hasher = Sha1::new();
    hasher.update(&payload);

    if hash != format!("{:x}", hasher.finalize()) {
        debug!("Actual resource hash doesn't match hash in request");
        return Err(error::ErrorBadRequest(""));
    }

    // TODO: add more checks n shit
    
    let path = get_res_path(&config.resource_dir, &hash);

    if path.exists() {
        debug!("Resource is already uploaded");
        return Err(error::ErrorConflict(""));
    }

    fs::create_dir_all(&config.resource_dir).map_err(|e| {
        error!("Couldn't create resource dir: {e}");
        error::ErrorInternalServerError("")
    })?;
    fs::write(path, &payload).map_err(|e| {
        error!("Couldn't write to resource file: {e}");
        error::ErrorInternalServerError("")
    })?;

    Ok(HttpResponse::Ok())
}

#[derive(Deserialize)]
pub struct ResourceList {
    #[serde(default)]
    resource: Vec<String>,
}

pub async fn filter_resources(payload: actix_xml::Xml<ResourceList>, config: web::Data<Config>) -> Result<impl Responder> {{
    Ok(Xml(xml!(
        resources {
            @for hash in &payload.resource {
                @let hash = hash.to_ascii_lowercase();
                @if !check_sha1(&hash) || !get_res_path(&config.resource_dir, &hash).exists() {
                    resource { (hash) }
                }
            }
        }
    ).into_string()))
}}