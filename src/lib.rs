use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(get_health))
            .route("/subscribe", web::post().to(post_subscribe))
    })
    .listen(listener)?
    .run();
    Ok(server)
}

async fn get_health() -> impl Responder {
    HttpResponse::Ok()
}

#[derive(serde::Deserialize)]
struct SubscribeFormData {
    name: String,
    email: String,
}

async fn post_subscribe(_form: web::Form<SubscribeFormData>) -> impl Responder {
    HttpResponse::Ok()
}
