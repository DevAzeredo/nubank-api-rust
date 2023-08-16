mod api_nubank;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use api_nubank::controller::certificate_controller::create_certificate;
use api_nubank::controller::nubank_controller::{nubank_payment_request, payment_details, payment_request};
use reqwest::header::{HeaderValue, ACCEPT, CONTENT_TYPE, USER_AGENT};
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

use crate::api_nubank::{
    create_certificate, discover::Discovery, nubank_payment_details, save_certificate,
};
lazy_static::lazy_static! {
    static ref REQUEST_HEADERS:reqwest::header::HeaderMap = create_headers();
}
static DB: Surreal<Any> = Surreal::init();

fn create_headers() -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_static("NuRust"));
    headers.insert("X-Correlation-Id", HeaderValue::from_static("and-7-0-0"));
    headers
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    DB.connect("file://db.db").await.unwrap();
    DB.use_ns("namespace").use_db("database").await.unwrap();
    let mut discovery = Discovery::new();
    discovery.init().await;
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(discovery.clone()))
            .service(create_certificate)
            .service(save_certificate)
            .service(payment_request)
            .service(payment_details)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
