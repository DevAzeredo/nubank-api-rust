use serde::{Serialize, Deserialize};

use crate::{DB, api_nubank::{nubank_model::Nubank, nubank_view::Payment}};

pub async fn create(nu: Nubank) {
    let _created: Nubank = DB
        .create(("nubank", nu.login.clone()))
        .content(nu)
        .await
        .unwrap();
}
pub async fn update(nu: Nubank) {
    let _created: Nubank = DB
        .update(("nubank", nu.login.clone()))
        .content(nu)
        .await
        .unwrap();
}
pub async fn get(login: String) -> Nubank {
    let select: Nubank = DB.select(("nubank", login)).await.unwrap();
    select
}


#[derive(Debug, Serialize, Deserialize)]

struct PaymentRequestDAO {
    login: String,
    payment: Payment,
}

pub async fn create_payment(payment: Payment, login: String) {
    let paymentdb = PaymentRequestDAO { login, payment };
    let _created: PaymentRequestDAO = DB
        .create("nu_payment")
        .content(paymentdb)
        .await
        .unwrap();

}
