use anyhow::{anyhow, Result};
use openssl::error::ErrorStack as OpenSSLStackError;
use openssl::pkcs12::Pkcs12;
use openssl::{
    pkey::{PKey, Private},
    rsa::Rsa,
    x509::X509,
};
use regex::Regex;
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::send_request;
use crate::{
    api_nubank::{model::model_dao::certificate_dao, payload},
    REQUEST_HEADERS,
};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CertificateCrypto {
    certificate: String,
    certificate_crypto: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Certificate {
    pub encrypted_code: String,
    #[serde(
        serialize_with = "serialize_private_key",
        deserialize_with = "deserialize_private_key"
    )]
    pub key: PKey<Private>,
    #[serde(
        serialize_with = "serialize_private_key",
        deserialize_with = "deserialize_private_key"
    )]
    pub crypto_key: PKey<Private>,
    pub certificate_key: Option<Vec<u8>>,
    pub certificate_crypto: Option<Vec<u8>>,
}
pub async fn get_public_keys(
    private_key: &PKey<Private>,
    private_crypto: &PKey<Private>,
) -> Result<(String, String), OpenSSLStackError> {
    let public_key = get_public_key(&private_key)?;
    let public_crypto = get_public_key(&private_crypto)?;

    Ok((public_key, public_crypto))
}

pub async fn request_code_email(
    login: String,
    password: String,
    device_id: String,
    url: String,
) -> Result<String, anyhow::Error> {
    let mut cert = Certificate::new();
    let (public_key, public_crypto) = get_public_keys(&cert.key, &cert.crypto_key).await?;

    let payload = payload::get_create_cert(
        login.clone(),
        password,
        device_id,
        public_key,
        public_crypto,
    );
    let response = match send_request(Method::POST, url, &payload).await {
        Ok(res) => res,
        Err(err) => {
            return Err(anyhow!("Response error during request code email {:?}", err));
        }
    };

    let header_str = response
        .headers()
        .get("www-authenticate")
        .ok_or_else(|| anyhow!("No 'www-authenticate' header"))?
        .to_str()
        .map_err(|_| anyhow!("Error parsing header value"))?;

    if header_str.is_empty() {
        return Err(anyhow!("Empty header value"));
    }

    let email = extract_sent_to(header_str).ok_or_else(|| anyhow!("Error extracting email"))?;

    cert.encrypted_code = extract_device_authorization_code(header_str)
        .ok_or_else(|| anyhow!("Error extracting device authorization code"))?;

    certificate_dao::create(cert, login).await;

    Ok(email)
}
pub async fn exchange(
    mut cert: Certificate,
    code: &str,
    login: String,
    password: String,
    device_id: String,
    url: String,
) -> Result<Certificate, anyhow::Error> {
    if cert.encrypted_code.is_empty() {
        return Err(anyhow!("Encrypted code not found"));
    }

    let (public_key, public_crypto) =
        match (get_public_key(&cert.key), get_public_key(&cert.crypto_key)) {
            (Ok(public), Ok(crypto)) => (public, crypto),
            error => {
                return Err(anyhow!("Fail to get public key {:?}", error));
            }
        };

    let mut payload =
        payload::get_create_cert(login, password, device_id, public_key, public_crypto);
    payload.insert("code".to_owned(), code.to_string());
    payload.insert("encrypted-code".to_owned(), cert.encrypted_code.clone());

    let response = match send_request(Method::POST, url, &payload).await {
        Ok(res) => res,
        Err(_err) => {
            return Err(anyhow!("Response error during exchange certificate"));
        }
    };

    let data: CertificateCrypto = match response.json().await {
        Ok(result) => result,
        Err(error) => {
            return Err(anyhow!("Certificate Crypto Error {:?}", error));
        }
    };

    (cert.certificate_key, cert.certificate_crypto) =
        gen_certificates(&cert.key, &cert.crypto_key, data).await?;

    Ok(cert)
}

pub async fn gen_certificates(
    key: &PKey<Private>,
    crypto_key: &PKey<Private>,
    data: CertificateCrypto,
) -> Result<(Option<Vec<u8>>, Option<Vec<u8>>), anyhow::Error> {
    let (key_x, crypto_x) = match (
        parse_cert(&data.certificate),
        parse_cert(&data.certificate_crypto),
    ) {
        (Ok(certificate1), Ok(certificate2)) => (certificate1, certificate2),
        error => {
            return Err(anyhow!("Error during parse certificate {:?}", error));
        }
    };
    let key_pkcs12 = match gen_cert(&key, &key_x) {
        Ok(result) => result,
        Err(error) => {
            return Err(anyhow!("Fail to generate key pkcs {:?}", error));
        }
    };

    let crypto_pkcs12 = match gen_cert(&crypto_key, &crypto_x) {
        Ok(result) => result,
        Err(error) => {
            return Err(anyhow!("Fail to generate crypto pkcs {:?}", error));
        }
    };

    let cetificate_key = match key_pkcs12.to_der() {
        Ok(der_data) => der_data,
        Err(error) => {
            return Err(anyhow!("Fail to generate key certificate{:?}", error));
        }
    };
    let certificate_crypto = match crypto_pkcs12.to_der() {
        Ok(der_data) => der_data,
        Err(error) => {
            return Err(anyhow!("Fail to generate crypto certificate {:?}", error));
        }
    };

    Ok((Some(cetificate_key), Some(certificate_crypto)))
}

impl Default for Certificate {
    fn default() -> Self {
        Certificate::new()
    }
}

impl Certificate {
    pub fn new() -> Certificate {
        let key: PKey<Private> = generate_key().unwrap();
        let crypto_key = generate_key().unwrap();
        let encrypted_code = "".to_string();

        Certificate {
            encrypted_code,
            key,
            crypto_key,
            certificate_key: None,
            certificate_crypto: None,
        }
    }
}

fn parse_cert(content: &str) -> Result<X509, OpenSSLStackError> {
    let cert = X509::from_pem(content.as_bytes())?;
    Ok(cert)
}

fn gen_cert(key: &PKey<Private>, cert: &X509) -> Result<Pkcs12, OpenSSLStackError> {
    let mut builder = Pkcs12::builder();
    builder.name("password");
    builder.pkey(key);
    builder.cert(cert);
    let p12 = builder.build2("NuRust")?;

    Ok(p12)
}
pub fn get_public_key(private_key: &PKey<Private>) -> Result<String, OpenSSLStackError> {
    let rsa = private_key.rsa()?;
    let public_key = PKey::from_rsa(rsa)?;

    let public_key_pem = match public_key.public_key_to_pem() {
        Ok(pem) => pem,
        Err(_) => {
            return Err(OpenSSLStackError::get());
        }
    };

    let public_key_str = match String::from_utf8(public_key_pem) {
        Ok(str) => str,
        Err(_) => {
            return Err(OpenSSLStackError::get());
        }
    };

    Ok(public_key_str)
}
fn generate_key() -> Result<PKey<Private>, Box<dyn std::error::Error>> {
    let rsa = Rsa::generate(2048)?;
    let private_key = PKey::from_rsa(rsa)?;

    Ok(private_key)
}
fn extract_device_authorization_code(header: &str) -> Option<String> {
    let re = Regex::new(r#"device-authorization encrypted-code="([^"]+)""#).unwrap();
    if let Some(capture) = re.captures(header) {
        return Some(capture[1].to_string());
    }
    None
}
fn extract_sent_to(header: &str) -> Option<String> {
    let re = Regex::new(r#"sent-to=\"([^\"\\]*)\""#).unwrap();
    if let Some(capture) = re.captures(header) {
        return Some(capture[1].to_string());
    }
    None
}
pub fn serialize_private_key<S>(key: &PKey<Private>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let pem = key.private_key_to_pem_pkcs8().unwrap();
    let pem_str = String::from_utf8_lossy(&pem);
    serializer.serialize_str(&pem_str)
}

pub fn deserialize_private_key<'de, D>(deserializer: D) -> Result<PKey<Private>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let pem_str: String = serde::Deserialize::deserialize(deserializer)?;
    let pem = pem_str.as_bytes();
    let key = PKey::private_key_from_pem(pem).unwrap();
    Ok(key)
}
