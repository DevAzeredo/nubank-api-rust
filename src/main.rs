mod api_nubank;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

use actix_web::web::Data;
use actix_web::{App, HttpServer};
use api_nubank::controller::certificate_controller::{self, create_certificate, save_certificate};
use api_nubank::controller::nubank_controller::{payment_details, payment_request};
use api_nubank::discover::Discovery;
use lazy_static::lazy_static;
use reqwest::header::{HeaderValue, ACCEPT, CONTENT_TYPE, USER_AGENT, AUTHORIZATION};
use reqwest::{Client, ClientBuilder, Method};
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

lazy_static! {
    static ref REQUEST_HEADERS: reqwest::header::HeaderMap = create_headers();
}

static DB: Surreal<Any> = Surreal::init();

fn create_headers(token: Option<&str>) -> Result<reqwest::header::HeaderMap, anyhow::Error> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_static("NuRust"));
    headers.insert("X-Correlation-Id", HeaderValue::from_static("and-7-0-0"));
    if token.is_some() {
        let authorization_value = match HeaderValue::from_str(&format!("Bearer {}", token)) {
            Ok(it) => it,
            Err(err) => return Err(anyhow!("Error to create headers {:?}", err)),
        };
        headers.insert(AUTHORIZATION, authorization_value);
    }
    headers
}
async fn get_client_with_identity(
    login: String,
    token: Option<&str>,
) -> Result<Client, anyhow::Error> {
    let certificate = certificate_controller::get_by_login(login.clone()).await;
    let cert_clone = &certificate.certificate_key.clone();
    let cert1 = match cert_clone {
        Some(certificate1) => certificate1,
        None => return Err(anyhow!("Error certificate not found")),
    };

    let identity = match reqwest::Identity::from_pkcs12_der(cert1, "NuRust") {
        Ok(identity) => identity,
        Err(_error) => return Err(anyhow!("Error to generate identity")),
    };

    let client = match ClientBuilder::new()
        .default_headers(create_headers(token.unwrap_or(&"".to_string()))?)
        .identity(identity)
        .build()
    {
        Ok(it) => it,
        Err(_err) => return Err(anyhow!("Error to build client")),
    };
    Ok(client)
}
pub async fn send_request(
    method: Method,
    url: String,
    payload: &HashMap<std::string::String, std::string::String>,
) -> Result<reqwest::Response, reqwest::Error> {
    Client::new()
        .request(method, url)
        .headers(REQUEST_HEADERS.clone())
        .json(&payload)
        .send()
        .await
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Inicialização do DB e definição de log
    DB.connect("file://db.db").await.unwrap();
    DB.use_ns("namespace").use_db("database").await.unwrap();

    // Inicialização do sistema de descoberta e log
    let mut discovery = Discovery::new();
    discovery.init().await;
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    // Configuração do servidor HTTP
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
