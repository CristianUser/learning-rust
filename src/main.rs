use actix_cors::Cors;
use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[path = "printing/routes.rs"]
mod routes;

use routes::{list_printers, print};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct Health {
    status: String,
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(Health { status: "ok".to_owned() })
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let server = HttpServer::new(|| {
        let cors = Cors::permissive(); // Change this to configure your CORS settings

        App::new()
            .wrap(cors)
            .service(health)
            .service(list_printers)
            .service(print)
    })
    .bind(("127.0.0.1", 1829))?;

    println!("Listening on port 1829");
    server.run().await
}
