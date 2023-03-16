use actix_web::{Responder, web};
use maud::html;

use crate::responder::Xml;

pub async fn login(npticket: web::Bytes) -> impl Responder {
    // TODO: implement actual login



    Xml(
        html! {
            loginResult {
                authTicket { "MM_AUTH=fuck" }
                lbpEnvVer { "sacklite" }
            }
        }.into_string()
    )
}