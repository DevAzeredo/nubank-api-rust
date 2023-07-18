use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentResponsePayload {
    pub id: String,
    pub amount: f32,
    pub message: Option<String>,
    pub url: String,
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    #[serde(rename = "pixAlias")]
    pub pix_alias: String,
    pub brcode: String,
}

impl PaymentResponsePayload {
   pub fn new(message: String) -> Self {
    PaymentResponsePayload {
            amount: 0.0,
            id: "".to_string(),
            message: Some(message),
            brcode: "".to_string(),
            pix_alias: "".to_string(),
            transaction_id: "".to_string(),
            url: "".to_string(),
        }
    }
}
