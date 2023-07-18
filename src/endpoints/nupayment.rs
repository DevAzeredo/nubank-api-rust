use crate::{discover::Discovery, nubank::nubank_dao};
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PaymentCreatePayload {
    login: String,
    amount: f32,
}
#[actix_web::post("/payment/create")]
async fn nubank_payment_request(
    request: web::Json<PaymentCreatePayload>,
    disc: web::Data<Discovery>,
) -> impl Responder {
    let mut nu = nubank_dao::get_by_login(request.login.clone()).await;
    let ghost = match nu
        .authenticate_with_certificate(disc.proxy_list_app_url.token.clone())
        .await
    {
        Ok(res) => res,
        Err(error) => return HttpResponse::InternalServerError().body(error.to_string()),
    };

    disc.salvar_url_ghost_flame(ghost.clone()).await;

    match nu.get_pix_keys(ghost.clone()).await {
        Ok(res) => res,
        Err(error) => return HttpResponse::InternalServerError().body(error.to_string()),
    };

    let res = match nu
        .create_pix_qr_code(request.amount, ghost)
        .await
    {
        Ok(payment) => payment,
        Err(error) => return HttpResponse::InternalServerError().body(error.to_string()),
    };

    nubank_dao::update(nu).await;
    HttpResponse::Ok().json(res)
}
