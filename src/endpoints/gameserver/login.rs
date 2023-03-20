use actix_web::{Responder, web};
use log::debug;
use maud::html;

use crate::{
    responder::Xml,
    types::{npticket::NpTicket, pub_key_store::PubKeyStore},
};

pub async fn login(pub_key_store: web::Data<PubKeyStore>, npticket: web::Bytes) -> impl Responder {
    match NpTicket::parse_from_bytes(npticket) {
        Ok(t) => {
            debug!("ticket: {:#?}", t);
            debug!("ticket verified: {:?}", t.verify_signature(&pub_key_store));
        },
        Err(e) => debug!("NpTicket parsing failed: {:?}", e),
    };

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