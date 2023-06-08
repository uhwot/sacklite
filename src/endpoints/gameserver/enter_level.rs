use actix_web::web::Path;
use actix_web::{HttpResponse, Responder};

pub async fn enter_level(path: Path<(String, u64)>) -> impl Responder {
    let (slot_type, _) = path.into_inner();
    if !["developer", "user"].contains(&slot_type.as_str()) {
        return HttpResponse::NotFound();
    }
    HttpResponse::Ok()
}
