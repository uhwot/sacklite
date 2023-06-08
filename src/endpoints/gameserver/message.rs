use crate::types::Config;
use actix_web::web::Data;
use actix_web::{HttpResponse, Responder};

pub async fn eula(config: Data<Config>) -> String {
    config.eula.clone()
}

pub async fn announce(config: Data<Config>) -> String {
    config.announcement.clone()
}

pub async fn notification() -> impl Responder {
    HttpResponse::Ok()
}
