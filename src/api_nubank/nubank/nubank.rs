use std::error::Error;

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest::{header::AUTHORIZATION, Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::{REQUEST_HEADERS, api_nubank::{payload, payment::{payment::PaymentResponsePayload, payment_dao}, queries::{self, get_create_pix_qr_code, feed_items_query}, cert::certificate_dao}};

#[derive(Debug, Serialize, Deserialize)]
pub struct Nubank {
    pub login: String,
    pub password: String,
    pub device_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub pix: NubankPix,
    pub account_id: String,
}

impl Nubank {
    pub fn new(login: String, password: String, device_id: String) -> Self {
        Nubank {
            login,
            password,
            device_id,
            access_token: "".to_string(),
            refresh_token: "".to_string(),
            pix: NubankPix {
                formatted_value: "".to_string(),
                kind: "".to_string(),
                value: "".to_string(),
            },
            account_id: "".to_string(),
        }
    }
    pub async fn authenticate_with_certificate(
        &mut self,
        url: String,
    ) -> Result<String, Box<dyn Error>> {
        let client = self.get_client_with_identity().await?;

        let response = client
            .post(url)
            .headers(REQUEST_HEADERS.clone())
            .json(&payload::get_auth_cert(
                self.login.clone(),
                self.password.clone(),
            ))
            .send()
            .await?;

        let json: Value = serde_json::from_str(&response.text().await?)?;
        
        if let Some(access_token_href) = json["access_token"].as_str() {
            self.access_token = access_token_href.to_string();
        } else {
            return Err("Access Token not found in the response".to_string().into());
        }

        if let Some(refresh_token_href) = json["refresh_token"].as_str() {
            self.refresh_token = refresh_token_href.to_string();
        } else {
            return Err("Refresh Token not found in the response".to_string().into());
        }

        if let Some(ghostflame_href) = json["_links"]["savings_account"]["href"].as_str() {
            Ok(ghostflame_href.to_string())
        } else {
            return Err("GhostFlame not found in the response".to_string().into());
        }
    }

    pub async fn get_pix_keys(&mut self, url: String) -> Result<bool, Box<dyn Error>> {
        let client = self.get_client_with_identity().await?;
        let token = format!("Bearer {}", self.access_token.clone());

        let response = client
            .post(url)
            .headers(REQUEST_HEADERS.clone())
            .header(AUTHORIZATION, token)
            .json(&queries::get_pix_keys())
            .send()
            .await?;

        let response: Value =
            serde_json::from_str(&response.text().await.unwrap_or("".to_string())).unwrap();
        let savings_account = &response["data"]["viewer"]["savingsAccount"];
        let first_key = &savings_account["dict"]["keys"][0];

        self.pix.formatted_value = first_key["formattedValue"]
            .as_str()
            .unwrap_or("")
            .to_string();
        self.pix.kind = first_key["kind"].as_str().unwrap_or("").to_string();
        self.pix.value = first_key["value"].as_str().unwrap_or("").to_string();
        self.account_id = savings_account["id"].as_str().unwrap_or("").to_string();

        Ok(true)
    }

    pub async fn create_pix_qr_code(
        &mut self,
        amount: f32,
        url: String,
    ) -> Result<PaymentResponsePayload, Box<dyn Error>> {
        let client = self.get_client_with_identity().await?;
        let token = format!("Bearer {}", self.access_token.clone());
        let response = client
            .post(url)
            .headers(REQUEST_HEADERS.clone())
            .header(AUTHORIZATION, token)
            .json(&get_create_pix_qr_code(
                amount,
                self.pix.value.clone(),
                self.account_id.clone(),
                generate_transaction_id(),
            ))
            .send()
            .await?;

        let respond: serde_json::Value =
            serde_json::from_str(&response.text().await.unwrap_or("".to_string()))?;

        let payment_request_json = &respond["data"]["createPaymentRequest"]["paymentRequest"];

        let payment_request: PaymentResponsePayload =
            serde_json::from_value(payment_request_json.clone()).unwrap_or(
                PaymentResponsePayload::new("Error to Request Payment".to_owned()),
            );
        let res = payment_request.clone();
        if !payment_request.id.is_empty() {
            payment_dao::nubank_create(payment_request, self.login.clone()).await;
        }
        Ok(res)
    }

    pub async fn get_pix_by_identifier(
        &mut self,
        id: String,
        url: String,
    ) -> Result<String, Box<dyn Error>> {
        let token = format!("Bearer {}", self.access_token.clone());
        let client = self.get_client_with_identity().await?;
        let response = client
            .post(url)
            .headers(REQUEST_HEADERS.clone())
            .header(AUTHORIZATION, token)
            .json(&feed_items_query(""))
            .send()
            .await?;

        let respond: serde_json::Value =
            serde_json::from_str(&response.text().await.unwrap_or("".to_string()))?;

        if let Some(node) = find_node_by_id(&respond, &id) {
            Ok(json!(node).to_string())
        } else {
            return Err("Not found".to_string().into());
        }
    }
    async fn get_client_with_identity(&self) -> Result<Client, Box<dyn Error>> {
        let certificate = certificate_dao::get_by_login(self.login.clone()).await;
        let identity = reqwest::Identity::from_pkcs12_der(
            &certificate
                .cert1
                .clone()
                .ok_or("Error: Certificate not found")?,
            "NuRust",
        )
        .map_err(|err| format!("Fail to create identity: {}", err))?;
        let client = ClientBuilder::new().identity(identity).build()?;
        Ok(client)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NubankPix {
    pub formatted_value: String,
    pub kind: String,
    pub value: String,
}
fn generate_transaction_id() -> String {
    let rng = thread_rng();
    let transaction_id: String = rng
        .sample_iter(&Alphanumeric)
        .map(char::from)
        .take(12)
        .collect();

    transaction_id
}

fn find_node_by_id<'a>(json_value: &'a Value, id: &'a str) -> Option<&'a Map<String, Value>> {
    if let Some(nodes) =
        json_value["data"]["viewer"]["savingsAccount"]["feedItems"]["edges"].as_array()
    {
        for edge in nodes {
            if let Some(node) = edge["node"].as_object() {
                if let Some(node_id) = node["id"].as_str() {
                    if node_id == id {
                        return Some(node);
                    }
                }
            }
        }
    }

    None
}
