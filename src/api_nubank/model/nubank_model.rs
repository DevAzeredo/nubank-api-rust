use anyhow::{anyhow, Result};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Method,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::{
    api_nubank::{
        payload,
        queries::{self, feed_items_query, get_create_pix_qr_code},
        salvar_url_ghost_flame,
    },
    send_request, REQUEST_HEADERS,
};

use super::model_dao::nubank_dao;

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
    pub fn new(login: String, password: String) -> Self {
        Nubank {
            login,
            password,
            device_id: generate_random_id(),
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
    ) -> Result<String, anyhow::Error> {
        let client = get_client_with_identity(self.login.clone(), None).await?;

        let payload = payload::get_auth_cert(self.login.clone(), self.password.clone());

        let json_value: Value = match send_request(Method::POST, url, &payload).await {
            Ok(response) => {
                let json_string = response.text().await.unwrap_or("".to_string());
                serde_json::from_str(&json_string).unwrap_or_default()
            }
            Err(err) => {
                return Err(anyhow!(
                    "Response error during authenticate certificate {:?}",
                    err
                ));
            }
        };
        let (access_token, refresh_token, ghostflame_href) = (
            json_value["access_token"].as_str(),
            json_value["refresh_token"].as_str(),
            json_value["_links"]["savings_account"]["href"].as_str(),
        );

        if let (Some(access_token_href), Some(refresh_token_href), Some(ghostflame_href)) =
            (access_token, refresh_token, ghostflame_href)
        {
            self.access_token = access_token_href.to_string();
            self.refresh_token = refresh_token_href.to_string();
            Ok(ghostflame_href.to_string())
        } else {
            Err(anyhow!("Fail during Authentication with certificate"))
        }
    }

    pub async fn get_pix_keys(&mut self, url: String) -> Result<bool, anyhow::Error> {
        let token = format!("Bearer {}", self.access_token.clone());
        let client = get_client_with_identity(self.login.clone(), Some(&token)).await?;

        let payload = &queries::get_pix_keys();

        let (savings_account, first_key) = match client.post(url).json(&payload).send().await {
            Ok(response) => {
                let json_string = response.text().await.unwrap_or("".to_string());
                let json_value: Value = serde_json::from_str(&json_string).unwrap_or_default();
                let account = json_value["data"]["viewer"]["savingsAccount"].clone();
                let keys = account["dict"]["keys"][0].clone();
                (account, keys)
            }
            Err(_) => (
                serde_json::Value::default().clone(),
                serde_json::Value::default().clone(),
            ),
        };

        self.pix.formatted_value = first_key["formattedValue"]
            .as_str()
            .unwrap_or("")
            .to_string();
        self.pix.kind = first_key["kind"].as_str().unwrap_or("").to_string();
        self.pix.value = first_key["value"].as_str().unwrap_or("").to_string();
        self.account_id = savings_account["id"].as_str().unwrap_or("").to_string();

        Ok(true)
    }

    pub async fn get_pix_by_identifier(
        &mut self,
        id: String,
        url: String,
    ) -> Result<String, anyhow::Error> {
        let client =
            get_client_with_identity(self.login.clone(), Some(&self.access_token.clone())).await?;
        let payload = &feed_items_query("");

        let res = match client.post(url).json(&payload).send().await {
            Ok(response) => {
                let json_string = response.text().await.unwrap_or("".to_string());
                serde_json::from_str(&json_string).unwrap_or_default()
            }
            Err(_) => serde_json::Value::default().clone(),
        };

        if let Some(node) = find_node_by_id(&res, &id) {
            Ok(json!(node).to_string())
        } else {
            return Err(anyhow!("Error to find node"));
        }
    }
}
fn generate_random_id() -> String {
    let rng = rand::thread_rng();
    let id: String = rng
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();

    id
}

pub async fn create_pix_qr_code(
    login: String,
    amount: f32,
    url: String,
) -> Result<Value, anyhow::Error> {
    let mut nu = nubank_dao::get(login.clone()).await;
    let ghost = nu.authenticate_with_certificate(url.clone()).await?;

    salvar_url_ghost_flame(ghost.clone()).await;

    nu.get_pix_keys(ghost.clone()).await?;

    let payload = &get_create_pix_qr_code(
        amount,
        nu.pix.value.clone(),
        nu.account_id.clone(),
        generate_transaction_id(),
    );
    let response = get_client_with_identity(login, Some(&nu.access_token.clone()))
        .await?
        .post(url)
        .json(payload)
        .send()
        .await?;

    nubank_dao::update(nu).await;

    let res: serde_json::Value =
        serde_json::from_str(&response.text().await.unwrap_or("".to_string()))?;

    Ok(res["data"]["createPaymentRequest"]["paymentRequest"].clone())
}

fn create_headers(token: &str) -> Result<HeaderMap, anyhow::Error> {
    let mut res = REQUEST_HEADERS.clone();
    if !token.is_empty() {
        let authorization_value = match HeaderValue::from_str(&format!("Bearer {}", token)) {
            Ok(it) => it,
            Err(err) => return Err(anyhow!("Error to fcreate headers {:?}", err)),
        };
        res.insert(AUTHORIZATION, authorization_value);
    }
    Ok(res)
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
