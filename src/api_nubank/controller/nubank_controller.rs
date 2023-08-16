use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

use crate::api_nubank::{
    discover::Discovery,
    get_url_ghost_flame, nubank_dao,
    nubank_model::{self, Nubank},
    nubank_view,
};

#[derive(Debug, Deserialize)]
struct PaymentCreatePayload {
    login: String,
    amount: f32,
}
#[actix_web::post("/payment/create")]
async fn payment_request(
    request: web::Json<PaymentCreatePayload>,
    disc: web::Data<Discovery>,
) -> impl Responder {
    let payment = match nubank_model::create_pix_qr_code(
        request.login.clone(),
        request.amount,
        get_url_ghost_flame().await.unwrap(),
    )
    .await
    {
        Ok(payment) => payment,
        Err(error) => return HttpResponse::InternalServerError().body(error.to_string()),
    };

    match nubank_view::render_payment_response(payment) {
        Ok(response) => {
            if !response.id.is_empty() {
                nubank_dao::create_payment(response.clone(), request.login.clone()).await;
            }

            HttpResponse::Ok().json(response)
        }
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}

#[derive(Debug, Deserialize)]
struct PaymentDetailsPayload {
    login: String,
    id: String,
}
#[actix_web::get("/payment/details")]
async fn payment_details(
    request: web::Json<PaymentDetailsPayload>,
    disc: web::Data<Discovery>,
) -> impl Responder {
    let mut nu = nubank_dao::get(request.login.clone()).await;
    let url = get_url_ghost_flame().await.unwrap_or("".to_string());
    if !url.is_empty() {
        let res = match nu.get_pix_by_identifier(request.id.clone(), url).await {
            Ok(payment) => payment,
            Err(error) => return HttpResponse::InternalServerError().body(error.to_string()),
        };
        HttpResponse::Ok().json(res)
    } else {
        return HttpResponse::InternalServerError().body("URL ghost_flame not found");
    }
}

pub async fn create_nubank(login: String, password: String) -> nubank_model::Nubank {
    nubank_model::Nubank::new(login, password)
}
pub async fn insert_nubank(nu: Nubank) {
    nubank_dao::create(nu).await;
}
pub async fn get(login: String) -> nubank_model::Nubank {
    nubank_dao::get(login).await
}
