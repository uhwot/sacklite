use actix_web::{Responder, web};
use log::debug;
use maud::html;

use crate::{
    responder::Xml,
    types::npticket::NpTicket,
};

pub async fn login(npticket: web::Bytes) -> impl Responder {
    debug!("{:#?}", NpTicket::parse_from_bytes(npticket));

    // TODO: verify signature

    Xml(
        html! {
            loginResult {
                authTicket { "MM_AUTH=fuck" }
                lbpEnvVer { "sacklite" }
            }
        }.into_string()
    )
}