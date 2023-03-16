use actix_web::web;
use actix_web::{Responder, HttpResponse};
use crate::types::config::Config;

pub async fn eula(data: web::Data<Config>) -> String {
    data.eula.clone()
}

pub async fn announce(data: web::Data<Config>) -> String {
    data.announcement.clone()
}

pub async fn notification() -> impl Responder {
    HttpResponse::Ok()
}