use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;

#[path = "printing/routes.rs"] mod routes;

use routes::{list_printers, print};


#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = HttpServer::new(|| {
        let cors = Cors::permissive(); // Change this to configure your CORS settings

        App::new()
            .wrap(cors)
            .service(health)
            .service(list_printers)
            .service(print)
    })
    .bind(("127.0.0.1", 8080))?;
    
    println!("Listening on port 8080");
    server.run()
    .await
}
