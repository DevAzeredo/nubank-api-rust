use actix_web::{
    web::{self, Data},
    HttpResponse, Responder,
};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use crate::api_nubank::{cert::{certificate::Certificate, certificate_dao}, nubank::{nubank::Nubank, nubank_dao}, discover::Discovery};


#[derive(Debug, Deserialize)]
struct CertificateRequest {
    login: String,
    password: String,
}
#[derive(Debug, Serialize)]
struct CertificateResponse {
    message: String,
    email: String,
}

#[actix_web::post("/certificate/create")]
async fn create_certificate(
    request: web::Json<CertificateRequest>,
    disc: Data<Discovery>,
) -> impl Responder {
    let mut certificate = Certificate::new();
    let nu = Nubank::new(
        request.login.clone(),
        request.password.clone(),
        generate_random_id(),
    );
    let email = certificate
        .request_code_email(
            request.login.clone(),
            request.password.clone(),
            nu.device_id.clone(),
            disc.proxy_list_app_url.gen_certificate.clone(),
        )
        .await;
    certificate_dao::create(certificate, request.login.clone()).await;
    nubank_dao::create(nu).await;

    let response = match email {
        Ok(email_string) => CertificateResponse {
            message: "Success".to_string(),
            email: email_string,
        },
        Err(err) => CertificateResponse {
            message: err.to_string(),
            email: "".to_owned(),
        },
    };
    HttpResponse::Ok().json(response)
}

fn generate_random_id() -> String {
    let rng = rand::thread_rng();
    let id: String = rng
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();

    id
}
