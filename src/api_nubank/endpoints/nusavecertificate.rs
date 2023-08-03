use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

use crate::api_nubank::{discover::Discovery, cert::certificate_dao, nubank::nubank_dao};


#[derive(Debug, Deserialize)]
struct CodeCertificateRequest {
    login: String,
    code: String,
}
#[actix_web::post("/certificate/save")]
async fn save_certificate(
    request: web::Json<CodeCertificateRequest>,
    disc: web::Data<Discovery>,
) -> impl Responder {
    let mut certificate = certificate_dao::get_by_login(request.login.clone()).await;
    let nubank = nubank_dao::get_by_login(request.login.clone()).await;
    let code = request.code.clone();
    match certificate
        .exchange(
            &code,
            nubank.login.clone(),
            nubank.password.clone(),
            nubank.device_id.clone(),
            disc.proxy_list_app_url.gen_certificate.clone(),
        )
        .await
    {
        Ok(_result) => {
            certificate_dao::update(certificate, nubank.login.clone()).await;
            nubank_dao::update(nubank).await;
            HttpResponse::Ok().finish()
        }
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}
