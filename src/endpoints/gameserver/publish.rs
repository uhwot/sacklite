use std::sync::Arc;

use actix_web::{
    error,
    web::{Data, ReqData, Path},
    Responder, Result, HttpResponse,
};
use maud::html as xml;
use serde::Deserialize;
use serde_with::{serde_as, BoolFromInt, DisplayFromStr, StringWithSeparator, formats::CommaSeparator};
use sqlx::{Pool, Postgres};

use crate::{
    responder::Xml,
    types::{Config, SessionData, ResourceRef},
};

use super::Location;

#[serde_as]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotPublishPayload {
    id: Option<i64>,
    name: String,
    description: String,
    #[serde_as(as = "DisplayFromStr")]
    icon: ResourceRef,
    #[serde_as(as = "serde_with::hex::Hex")]
    root_level: [u8; 20],
    #[serde(default)]
    #[serde_as(as = "Vec<serde_with::hex::Hex>")]
    resource: Vec<[u8; 20]>,
    location: Location,
    initially_locked: bool,
    #[serde(default)]
    is_sub_level: bool,
    #[serde(default)]
    #[serde(rename = "isLBP1Only")]
    is_lbp1_only: bool,
    #[serde_as(as = "BoolFromInt")]
    shareable: bool,
    leveltype: String,
    min_players: u8,
    max_players: u8,
    #[serde_as(as = "Option<StringWithSeparator::<CommaSeparator, String>>")]
    labels: Option<Vec<String>>,
    #[serde(default)]
    move_required: bool,
    #[serde(default)]
    vita_cross_control_required: bool,
}

pub async fn start_publish(
    payload: actix_xml::Xml<SlotPublishPayload>,
    config: Data<Config>,
) -> Result<impl Responder> {
    let mut resources: Vec<ResourceRef> = payload.resource.iter().map(|r| ResourceRef::Hash(*r)).collect();
    resources.push(payload.icon.clone());

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

    if pl.id.is_none() {
        let num_slots = sqlx::query!("SELECT COUNT(*) AS num_slots FROM slots WHERE author = $1", session.user_id)
        .fetch_one(&***pool)
        .await
        .map_err(error::ErrorInternalServerError)?
        .num_slots
        .unwrap_or_default();

        if num_slots >= config.slot_limit.into() {
            return Err(error::ErrorUnauthorized("User has reached slot limit"));
        }
    }

    let mut resources: Vec<ResourceRef> = pl.resource.iter().map(|r| ResourceRef::Hash(*r)).collect();
    resources.push(pl.icon.clone());
    resources.push(ResourceRef::Hash(pl.root_level));
    for res in resources {
        if !res.exists(&config.resource_dir) {
            return Err(error::ErrorBadRequest("One or more resources don't exist"));
        }
    }

    // TODO: add checks based on game version

    let res_array: Vec<String> = pl.resource.iter().map(hex::encode).collect();

    let slot_id = match pl.id {
        None => sqlx::query!(
            "INSERT INTO slots (
                name, author, description, icon, gamever, root_level, resources, location_x, location_y,
                initially_locked, is_sub_level, is_lbp1_only, shareable, level_type,
                min_players, max_players, move_required, vita_cc_required
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING id",
            pl.name,
            session.user_id,
            pl.description,
            pl.icon.to_string(),
            session.game_version as i16,
            hex::encode(pl.root_level),
            res_array.as_slice(),
            pl.location.x as i32,
            pl.location.y as i32,
            pl.initially_locked,
            pl.is_sub_level,
            pl.is_lbp1_only,
            pl.shareable,
            pl.leveltype,
            pl.min_players as i16,
            pl.max_players as i16,
            pl.move_required,
            pl.vita_cross_control_required
        )
        .fetch_one(&***pool)
        .await
        .map_err(error::ErrorInternalServerError)?
        .id,
        Some(id) => {sqlx::query!(
            "UPDATE slots
            SET name=$1, description=$2, icon=$3, gamever=$4, root_level=$5, resources=$6, location_x=$7, location_y=$8,
                initially_locked=$9, is_sub_level=$10, is_lbp1_only=$11, shareable=$12, level_type=$13,
                min_players=$14, max_players=$15, move_required=$16, vita_cc_required=$17,
                updated_at=CURRENT_TIMESTAMP
            WHERE id = $18 AND author = $19",
            pl.name,
            pl.description,
            pl.icon.to_string(),
            session.game_version as i16,
            hex::encode(pl.root_level),
            res_array.as_slice(),
            pl.location.x as i32,
            pl.location.y as i32,
            pl.initially_locked,
            pl.is_sub_level,
            pl.is_lbp1_only,
            pl.shareable,
            pl.leveltype,
            pl.min_players as i16,
            pl.max_players as i16,
            pl.move_required,
            pl.vita_cross_control_required,
            id,
            session.user_id
        )
        .execute(&***pool)
        .await
        .map_err(error::ErrorInternalServerError)?;
        id},
    };

    Ok(Xml(xml!(
        slot type="user" {
            id { (slot_id) }
        }
    ).into_string()))
}

pub async fn unpublish(
    path: Path<i64>,
    pool: Data<Arc<Pool<Postgres>>>,
    session: ReqData<SessionData>,
) -> Result<impl Responder> {
    let id = path.into_inner();

    let rows = sqlx::query!(
        "DELETE FROM slots WHERE id = $1 AND author = $2",
        id,
        session.user_id
    )
    .execute(&***pool)
    .await
    .map_err(error::ErrorInternalServerError)?
    .rows_affected();

    if rows == 0 {
        return Err(error::ErrorUnauthorized("Slot not found or not authorized to delete"));
    }

    Ok(HttpResponse::Ok())
}