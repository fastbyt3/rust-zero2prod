use actix_web::{HttpResponse, Responder};

pub async fn get_health() -> impl Responder {
    HttpResponse::Ok()
}
