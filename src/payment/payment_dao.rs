use serde::{Serialize, Deserialize};

use crate::{DB};

use super::payment::PaymentResponsePayload;
#[derive(Debug, Serialize, Deserialize)]
struct NubankPaymentRequestDB {
    login: String,
    payment: PaymentResponsePayload,
}

pub async fn nubank_create(payment: PaymentResponsePayload, login: String) {
    let paymentdb = NubankPaymentRequestDB { login, payment };
    let _created: NubankPaymentRequestDB = DB
        .create("nu_payment")
        .content(paymentdb)
        .await
        .unwrap();

}
