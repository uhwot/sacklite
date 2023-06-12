use actix_web::{web::Data, Result, Responder, error, HttpResponse};
use maud::html as xml;
use serde::Deserialize;
use serde_inline_default::serde_inline_default;

use crate::{types::Config, responder::Xml, utils::{resource::{check_sha1, get_res_path, res_exists}, deserialize}};

use super::Location;

#[serde_inline_default]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotPublishPayload {
    name: String,
    description: String,
    icon: String,
    root_level: String,
    #[serde(default)]
    resource: Vec<String>,
    location: Location,
    initially_locked: bool,
    #[serde_inline_default(false)]
    is_sub_level: bool,
    #[allow(non_snake_case)]
    #[serde_inline_default(false)]
    isLBP1Only: bool,
    #[serde(deserialize_with = "deserialize::bool_from_int")]
    shareable: bool,
    level_type: String,
    min_players: u8,
    max_players: u8,
    #[serde_inline_default(false)]
    move_required: bool,
    #[serde_inline_default(false)]
    vita_cross_control_required: bool,
}

pub async fn start_publish(payload: actix_xml::Xml<SlotPublishPayload>, config: Data<Config>) -> Result<impl Responder> {
    let mut resources: Vec<&String> = payload.resource.iter().map(|r| r).collect();
    resources.push(&payload.icon);
    resources.push(&payload.root_level);

    Ok(Xml(xml!(
        slot type="user" {
            @for resource in resources {
                @if check_sha1(&resource) && !get_res_path(&config.resource_dir, &resource).exists() {
                    resource { (resource) }
                }
            }
        }
    ).into_string()))
}

pub async fn publish(payload: actix_xml::Xml<SlotPublishPayload>, config: Data<Config>) -> Result<impl Responder> {
    if !res_exists(&config.resource_dir, &payload.root_level, true, true) {
        return Err(error::ErrorBadRequest(""))
    }
    {
        let mut resources: Vec<&String> = payload.resource.iter().map(|r| r).collect();
        resources.push(&payload.icon);
        for res in resources {
            if !res_exists(&config.resource_dir, res, true, false) {
                return Err(error::ErrorBadRequest(""))
            }
        }
    }
    
    // TODO: finish this lol

    Ok(HttpResponse::Ok())
}