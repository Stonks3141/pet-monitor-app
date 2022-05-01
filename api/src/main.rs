use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_files;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::fs::read_to_string;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body(read_to_string("static/index.html").unwrap())
}

#[get("/api/salt")]
async fn echo() -> impl Responder {
    HttpResponse::Ok().body(read_to_string("salt.dat").unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(
                actix_files::Files::new("/", "static")
                .prefer_utf8(true))
            .service(echo)
    })
    .bind_openssl("127.0.0.1:8080", builder)?
    .run()
    .await
}
