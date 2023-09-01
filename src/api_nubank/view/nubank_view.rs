use serde_json::Error;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Payment {
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

pub fn render_payment_response(payment_json: Value) -> Result<Payment, Error> {
    let payment_response = match serde_json::from_value::<Payment>(payment_json.clone()) {
        Ok(response) => response,
        Err(error) => return Err(error),
    };
    Ok(payment_response)
}

