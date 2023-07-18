use crate::{discover::Discovery, nubank::nubank_dao};
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PaymentDetailsPayload {
    login: String,
    id: String,
}
#[actix_web::get("/payment/details")]
async fn nubank_payment_details(
    request: web::Json<PaymentDetailsPayload>,
    disc: web::Data<Discovery>,
) -> impl Responder {
    let mut nu = nubank_dao::get_by_login(request.login.clone()).await;
    let url = disc.get_url_ghost_flame().await.unwrap_or("".to_string());
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
