use actix_web::web;
use actix_web::{Responder, HttpResponse};

pub async fn enter_level(path: web::Path<(String, u64)>) -> impl Responder {
    let (slot_type, _) = path.into_inner();
    if !["developer", "user"].contains(&slot_type.as_str()) {
        return HttpResponse::NotFound().finish();
    }
    HttpResponse::Ok().finish()
}