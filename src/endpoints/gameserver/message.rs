use actix_web::web;
use actix_web::{Responder, HttpResponse};
use crate::types::config::Config;

pub async fn eula(config: web::Data<Config>) -> String {
    config.eula.clone()
}

pub async fn announce(config: web::Data<Config>) -> String {
    config.announcement.clone()
}

pub async fn notification() -> impl Responder {
    HttpResponse::Ok()
}