use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;

use crate::routes::{get_health, post_subscribe};

pub fn run(listener: TcpListener, connection_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(connection_pool);
    let server = HttpServer::new(move || {
        App::new()
            .route("/health", web::get().to(get_health))
            .route("/subscribe", web::post().to(post_subscribe))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
