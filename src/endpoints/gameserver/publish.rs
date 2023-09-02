use std::sync::Arc;

use actix_web::{
    error,
    web::{Data, ReqData},
    HttpResponse, Responder, Result,
};
use maud::html as xml;
use serde::Deserialize;
use serde_with::{serde_as, BoolFromInt, DefaultOnError, DisplayFromStr};
use sqlx::{Pool, Postgres};

use crate::{
    responder::Xml,
    types::{Config, SessionData, ResourceRef},
};

use super::Location;

#[serde_as]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(non_snake_case)]
pub struct SlotPublishPayload {
    name: String,
    description: String,
    #[serde_as(as = "DefaultOnError<Option<DisplayFromStr>>")]
    icon: Option<ResourceRef>,
    #[serde_as(as = "serde_with::hex::Hex")]
    root_level: [u8; 20],
    #[serde(default)]
    #[serde_as(as = "Vec<DisplayFromStr>")]
    resource: Vec<ResourceRef>,
    location: Location,
    initially_locked: bool,
    #[serde(default)]
    is_sub_level: bool,
    #[serde(default)]
    isLBP1Only: bool,
    #[serde_as(as = "BoolFromInt")]
    shareable: bool,
    level_type: String,
    min_players: u8,
    max_players: u8,
    #[serde(default)]
    move_required: bool,
    #[serde(default)]
    vita_cross_control_required: bool,
}

pub async fn start_publish(
    payload: actix_xml::Xml<SlotPublishPayload>,
    config: Data<Config>,
) -> Result<impl Responder> {
    let mut resources = payload.resource.clone();
    if let Some(icon) = &payload.icon {
        resources.push(icon.clone());
    }
    resources.push(ResourceRef::Hash(payload.root_level));

    Ok(Xml(xml!(
        slot type="user" {
            @for resource in resources {
                @if !resource.exists(&config.resource_dir) {
                    resource { (resource.to_string()) }
                }
            }
        }
    ).into_string()))
}

pub async fn publish(
    pl: actix_xml::Xml<SlotPublishPayload>,
    pool: Data<Arc<Pool<Postgres>>>,
    config: Data<Config>,
    session: ReqData<SessionData>,
) -> Result<impl Responder> {
    for num_players in [pl.min_players, pl.max_players] {
        if ![1, 2, 3, 4].contains(&num_players) {
            return Err(error::ErrorBadRequest("Invalid player num"));
        }
    }
    if pl.max_players < pl.min_players {
        return Err(error::ErrorBadRequest(
            "Max players greater than min players",
        ));
    }

    let mut resources = pl.resource.clone();
    if let Some(icon) = &pl.icon {
        resources.push(icon.clone());
    }
    resources.push(ResourceRef::Hash(pl.root_level));
    for res in resources {
        if !res.exists(&config.resource_dir) {
            return Err(error::ErrorBadRequest("One or more resources don't exist"));
        }
    }

    // TODO: add checks based on game version

    let res_array: Vec<String> = pl.resource.iter().map(|r| r.to_string()).collect();

    sqlx::query!(
        "INSERT INTO slots (
            name, description, icon, gamever, root_level, resources, location_x, location_y,
            initially_locked, is_sub_level, is_lbp1_only, shareable, level_type,
            min_players, max_players, move_required, vita_cc_required
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)",
        pl.name,
        pl.description,
        pl.icon.as_ref().map(|r| r.to_string()),
        session.game_version as i16,
        hex::encode(pl.root_level),
        res_array.as_slice(),
        pl.location.x as i32,
        pl.location.y as i32,
        pl.initially_locked,
        pl.is_sub_level,
        pl.isLBP1Only,
        pl.shareable,
        pl.level_type,
        pl.min_players as i16,
        pl.max_players as i16,
        pl.move_required,
        pl.vita_cross_control_required
    )
    .execute(&***pool)
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok())
}
