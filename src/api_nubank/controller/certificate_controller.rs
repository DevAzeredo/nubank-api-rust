use actix_web::web::{self, Data};
use actix_web::{HttpResponse, Responder};
use serde::Deserialize;

use crate::api_nubank::certificate_view::{CertificateRequest, CertificateResponse};
use crate::api_nubank::controller::nubank_controller;
use crate::api_nubank::discover::Discovery;
use crate::api_nubank::model::certificate_model::{self, Certificate};
use crate::api_nubank::model::model_dao::certificate_dao;
use crate::api_nubank::nubank_dao;

#[actix_web::post("/certificate/create")]
async fn create_certificate(
    request: web::Json<CertificateRequest>,
    disc: Data<Discovery>,
) -> impl Responder {
    let nu =
        nubank_controller::create_nubank(request.login.clone(), request.password.clone()).await;

    let email = certificate_model::request_code_email(
        request.login.clone(),
        request.password.clone(),
        nu.device_id.clone(),
        disc.proxy_list_app_url.gen_certificate.clone(),
    )
    .await;

    nubank_controller::insert_nubank(nu).await;

    let res = match email {
        Ok(email_string) => HttpResponse::Ok().json(CertificateResponse {
            message: "Success".to_string(),
            email: email_string,
        }),
        Err(err) => HttpResponse::InternalServerError().json(CertificateResponse {
            message: err.to_string(),
            email: "".to_owned(),
        }),
    };
    res
}
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
    let certificate = certificate_dao::get(request.login.clone()).await;
    let nubank = nubank_controller::get(request.login.clone()).await;
    let code = request.code.clone();
    match certificate_model::exchange(
        certificate, 
        &code,
        nubank.login.clone(),
        nubank.password.clone(),
        nubank.device_id.clone(),
        disc.proxy_list_app_url.gen_certificate.clone(),
    )
    .await
    {
        Ok(exchanged_certificate) => {
            certificate_dao::update(exchanged_certificate, nubank.login.clone()).await;
            nubank_dao::update(nubank).await;
            HttpResponse::Ok().finish()
        }
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}

pub async fn get_by_login(login: String) -> Certificate {
    certificate_dao::get(login).await
}
