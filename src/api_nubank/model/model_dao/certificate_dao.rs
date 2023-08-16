use crate::{DB, api_nubank::model::certificate_model::{serialize_private_key, deserialize_private_key, Certificate}};

use openssl::pkey::{PKey, Private};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CertificateDB {
    pub login: String,
    pub password: String,
    pub device_id: String,
    pub encrypted_code: String,
    #[serde(
        serialize_with = "serialize_private_key",
        deserialize_with = "deserialize_private_key"
    )]
    pub key1: PKey<Private>,
    #[serde(
        serialize_with = "serialize_private_key",
        deserialize_with = "deserialize_private_key"
    )]
    pub key2: PKey<Private>,
    pub cert1: Option<Vec<u8>>,
    pub cert2: Option<Vec<u8>>,
}

pub(crate) async fn create(certificate: Certificate, login: String) {
    let _created: Certificate = DB
        .create(("certificate", login.clone()))
        .content(certificate)
        .await
        .unwrap_or_default();
}
pub(crate) async fn update(certificate: Certificate, login: String) {
    let _created: Certificate = DB
        .update(("certificate", login.clone()))
        .content(certificate)
        .await
        .unwrap_or_default();
}
pub(crate) async fn get(login: String) -> Certificate {
    let select: Certificate = DB.select(("certificate", login)).await.unwrap_or_default();
    select
}
