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
use std::fmt::Error;

use crate::{
    api_nubank::{model::model_dao::certificate_dao, payload},
    create_headers, REQUEST_HEADERS,
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
    pub key1: PKey<Private>,
    #[serde(
        serialize_with = "serialize_private_key",
        deserialize_with = "deserialize_private_key"
    )]
    pub key2: PKey<Private>,
    pub cert1: Option<Vec<u8>>,
    pub cert2: Option<Vec<u8>>,
}

pub async fn request_code_email(
    login: String,
    password: String,
    device_id: String,
    url: String,
) -> Result<String, Error> {
    let mut cert = Certificate::new();
    let (public_key, public_crypto) = match (get_public_key(&cert.key1), get_public_key(&cert.key2))
    {
        (Ok(public), Ok(crypto)) => (public, crypto),
        _ => {
            return Err(Error::default());
        }
    };

    let payload = payload::get_create_cert(
        login.clone(),
        password,
        device_id,
        public_key,
        public_crypto,
    );
    let response = match Client::new()
        .request(Method::POST, url)
        .headers(REQUEST_HEADERS.clone())
        .json(&payload)
        .send()
        .await
    {
        Ok(res) => res,
        Err(err) => {
            return Err(Error::default());
        }
    };
    match response.headers().get("www-authenticate") {
        Some(header_value) => {
            let header_str = header_value.to_str().unwrap_or_default();
            if !header_str.is_empty() {
                let email = match extract_sent_to(header_str).ok_or("Error extracting email") {
                    Ok(it) => it,
                    Err(err) => return Err(Error::default()),
                };
                cert.encrypted_code = extract_device_authorization_code(header_str).unwrap();
                certificate_dao::create(cert, login).await;
                return Ok(email);
            } else {
                return Err(Error::default());
            }
        }
        None => return Err(Error::default()),
    }
}
pub async fn exchange(
    mut cert: Certificate,
    code: &str,
    login: String,
    password: String,
    device_id: String,
    url: String,
) -> Result<String, Error> {
    if cert.encrypted_code.is_empty() {
        return Err(Error::default());
    }

    let (public_key, public_crypto) = match (get_public_key(&cert.key1), get_public_key(&cert.key2))
    {
        (Ok(public), Ok(crypto)) => (public, crypto),
        _ => {
            return Err(Error::default());
        }
    };

    let mut payload =
        payload::get_create_cert(login, password, device_id, public_key, public_crypto);
    payload.insert("code".to_owned(), code.to_string());
    payload.insert("encrypted-code".to_owned(), cert.encrypted_code.clone());
    let header = create_headers();

    let response = match Client::new()
        .post(url)
        .headers(header)
        .json(&payload)
        .send()
        .await
    {
        Ok(res) => res,
        Err(err) => {
            return Err(Error::default());
        }
    };

    let data: CertificateCrypto = match response.json().await {
        Ok(result) => result,
        Err(error) => {
            return Err(Error::default());
        }
    };
    let (cert1, cert2) = match (
        Ok(parse_cert(&data.certificate)),
        Ok(parse_cert(&data.certificate_crypto)),
    ) { PAREI AQUI TEM QUE DAR UMA ANALSIADA
        (Ok(cert1), Ok(cert2)) => (cert1, cert2),
        _ => {
            return Err(Error::default());
        }
    };

    let pkcs12_1 = match gen_cert(&cert.key1, &cert1) {
        Ok(result) => result,
        Err(_error) => {
            return Err(Error::default());
        }
    };

    let pkcs12_2 = match gen_cert(&cert.key2, &cert2) {
        Ok(result) => result,
        Err(_error) => {
            return Err(Error::default());
        }
    };

    match pkcs12_1.to_der() {
        Ok(der_data) => {
            cert.cert1 = Some(der_data);
        }
        Err(_error) => {
            return Err(Error::default());
        }
    }
    match pkcs12_2.to_der() {
        Ok(der_data) => {
            cert.cert2 = Some(der_data);
        }
        Err(_error) => {
            return Err(Error::default());
        }
    }
    let res = String::from("Certificates successfully exchanged!");
    Ok(res)
}

impl Default for Certificate {
    fn default() -> Self {
        Certificate::new()
    }
}

impl Certificate {
    pub fn new() -> Certificate {
        let key1: PKey<Private> = generate_key().unwrap();
        let key2 = generate_key().unwrap();
        let encrypted_code = "".to_string();

        Certificate {
            encrypted_code,
            key1,
            key2,
            cert1: None,
            cert2: None,
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
pub fn get_public_key(private_key: &PKey<Private>) -> Result<String, Box<dyn std::error::Error>> {
    let rsa = private_key.rsa()?;
    let public_key = PKey::from_rsa(rsa)?;
    let public_key_pem = public_key.public_key_to_pem()?;
    let public_key_str = String::from_utf8(public_key_pem)?;

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
