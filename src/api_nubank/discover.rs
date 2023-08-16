use crate::DB;
use reqwest::{Client, Result};
use serde::{Deserialize, Serialize};
use std::error::Error;

const DISCOVERY_URL: &str = "https://prod-global-webapp-proxy.nubank.com.br/api/discovery";
const DISCOVERY_APP_URL: &str = "https://prod-global-webapp-proxy.nubank.com.br/api/app/discovery";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Url {
    link: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Links {
    pub company_social_invite_by_slug: String,
    pub register_prospect_savings_web: String,
    pub register_prospect_savings_mgm: String,
    pub pusher_auth_channel: String,
    pub application_status_by_tax_id: String,
    pub register_prospect_debit: String,
    pub reset_password: String,
    pub register_prospect_ultraviolet_web: String,
    pub business_card_waitlist: String,
    pub lobby_offers: String,
    pub register_prospect: String,
    pub register_prospect_savings_request_money: String,
    pub register_prospect_global_web: String,
    pub register_prospect_c: String,
    pub request_password_reset: String,
    pub auth_gen_certificates: String,
    pub login: String,
    pub application_status_by_prospect_id: String,
    pub email_verify: String,
    pub register_prospect_company: String,
    pub get_customer_sessions: String,
    pub auth_device_resend_code: String,
    pub msat: String,
}
impl Default for Links {
    fn default() -> Self {
        Links {
            application_status_by_prospect_id: Default::default(),
            application_status_by_tax_id: Default::default(),
            auth_device_resend_code: Default::default(),
            auth_gen_certificates: Default::default(),
            business_card_waitlist: Default::default(),
            company_social_invite_by_slug: Default::default(),
            email_verify: Default::default(),
            get_customer_sessions: Default::default(),
            lobby_offers: Default::default(),
            login: Default::default(),
            msat: Default::default(),
            pusher_auth_channel: Default::default(),
            register_prospect: Default::default(),
            register_prospect_c: Default::default(),
            register_prospect_company: Default::default(),
            register_prospect_debit: Default::default(),
            register_prospect_global_web: Default::default(),
            register_prospect_savings_mgm: Default::default(),
            register_prospect_savings_request_money: Default::default(),
            register_prospect_savings_web: Default::default(),
            register_prospect_ultraviolet_web: Default::default(),
            request_password_reset: Default::default(),
            reset_password: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LinksAPP {
    pub unlogged_challenge_platform: String,
    pub scopes: String,
    pub creation: String,
    pub rosetta_images: String,
    pub change_password: String,
    pub smokejumper: String,
    pub block: String,
    pub lift: String,
    pub shard_mapping_id: String,
    pub foundation_tokens: String,
    pub application_status_by_tax_id: String,
    pub force_reset_password: String,
    pub rosetta_localization: String,
    pub revoke_token: String,
    pub userinfo: String,
    pub reset_password: String,
    pub lobby_offers: String,
    pub unblock: String,
    pub shard_mapping_cnpj: String,
    pub shard_mapping_cpf: String,
    pub register_prospect: String,
    pub engage: String,
    pub account_recovery_v2: String,
    pub send_data_to_etl: String,
    pub creation_with_credentials: String,
    pub magnitude: String,
    pub revoke_all: String,
    pub register_prospect_mobile_mgm_social: String,
    pub user_update_credential: String,
    pub user_hypermedia: String,
    pub gen_certificate: String,
    pub deferred_deeplink_application: String,
    pub application_status_by_prospect_id: String,
    pub email_verify: String,
    pub token: String,
    pub account_recovery: String,
    pub start_screen_v2: String,
    pub scopes_remove: String,
    pub approved_products: String,
    pub admin_revoke_all: String,
}
impl Default for LinksAPP {
    fn default() -> Self {
        LinksAPP {
            account_recovery: "".to_string(),
            account_recovery_v2: "".to_string(),
            admin_revoke_all: "".to_string(),
            application_status_by_prospect_id: "".to_string(),
            application_status_by_tax_id: "".to_string(),
            approved_products: "".to_string(),
            block: "".to_string(),
            change_password: "".to_string(),
            creation: "".to_string(),
            creation_with_credentials: "".to_string(),
            deferred_deeplink_application: "".to_string(),
            email_verify: "".to_string(),
            engage: "".to_string(),
            force_reset_password: "".to_string(),
            foundation_tokens: "".to_string(),
            gen_certificate: "".to_string(),
            lift: "".to_string(),
            lobby_offers: "".to_string(),
            magnitude: "".to_string(),
            register_prospect: "".to_string(),
            register_prospect_mobile_mgm_social: "".to_string(),
            reset_password: "".to_string(),
            revoke_all: "".to_string(),
            revoke_token: "".to_string(),
            rosetta_images: "".to_string(),
            rosetta_localization: "".to_string(),
            scopes: "".to_string(),
            scopes_remove: "".to_string(),
            send_data_to_etl: "".to_string(),
            shard_mapping_cnpj: "".to_string(),
            shard_mapping_cpf: "".to_string(),
            shard_mapping_id: "".to_string(),
            smokejumper: "".to_string(),
            start_screen_v2: "".to_string(),
            token: "".to_string(),
            unblock: "".to_string(),
            unlogged_challenge_platform: "".to_string(),
            user_hypermedia: "".to_string(),
            user_update_credential: "".to_string(),
            userinfo: "".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Discovery {
    client: Client,
    pub proxy_list_url: Links,
    pub proxy_list_app_url: LinksAPP,
}

impl Discovery {
    pub fn new() -> Self {
        let client = reqwest::Client::new();
        let proxy_list_url: Links = Links::default();
        let proxy_list_app_url = LinksAPP::default();
        Self {
            client,
            proxy_list_url,
            proxy_list_app_url,
        }
    }
    pub async fn init(&mut self) {
        self.proxy_list_url = Self::get_proxy_urls(&self.client).await.unwrap();
        self.proxy_list_app_url = Self::get_app_proxy_urls(&self.client).await.unwrap();
    }

    pub async fn get_proxy_urls(client: &reqwest::Client) -> Result<Links> {
        let response = client.get(DISCOVERY_URL).send().await?;
        let body = response.text().await?;
        let res: Links = serde_json::from_str(&body).unwrap();
        Ok(res)
    }
    pub async fn get_app_proxy_urls(client: &reqwest::Client) -> Result<LinksAPP> {
        let response = client.get(DISCOVERY_APP_URL).send().await?;
        let body = response.text().await?;
        let res: LinksAPP = serde_json::from_str(&body).unwrap();
        Ok(res)
    }
}
pub async fn salvar_url_ghost_flame(ghost_flame_url: String) {
    let _update: Url = DB
        .update(("url", "ghost_flame_url"))
        .content(Url {
            link: ghost_flame_url,
        })
        .await
        .unwrap();
}
pub async fn get_url_ghost_flame() -> core::result::Result<String, Box<dyn Error>> {
    let ghost_flame: Url = DB.select(("url", "ghost_flame_url")).await?;
    Ok(ghost_flame.link)
}
