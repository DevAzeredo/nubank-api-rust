use crate::api_nubank::payload;
use crate::{create_headers, REQUEST_HEADERS};
use log::error;
use openssl::error::ErrorStack as OpenSSLStackError;
use openssl::pkcs12::Pkcs12;
use openssl::pkey::{PKey, Private};
use openssl::rsa::Rsa;
use openssl::x509::X509;
use regex::Regex;
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use std::error::Error;

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
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ResCertificate {
    certificate: String,
    certificate_crypto: String,
}

impl Certificate {
    pub fn new() -> Self {
        let key1: PKey<Private> = Self::generate_key().unwrap();
        let key2 = Self::generate_key().unwrap();
        let encrypted_code = "".to_string();

        Certificate {
            encrypted_code,
            key1,
            key2,
            cert1: None,
            cert2: None,
        }
    }

    pub async fn request_code_email(
        &mut self,
        login: String,
        password: String,
        device_id: String,
        url: String,
    ) -> Result<String, Box<dyn Error>> {
        let payload = payload::get_create_cert(login, password, device_id, self);

        let response = match Client::new()
            .request(Method::POST, url)
            .headers(REQUEST_HEADERS.clone())
            .json(&payload)
            .send()
            .await
        {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to send request POST error: {err}");
                return Err("Failed to send request POST".into());
            }
        };
        match response.headers().get("www-authenticate") {
            Some(header_value) => {
                let header_str = header_value.to_str()?;
                let email = Self::extract_sent_to(header_str).ok_or("Error extracting email")?;
                self.encrypted_code = Self::extract_device_authorization_code(header_str).unwrap();

                Ok(email)
            }
            None => Err("'www-authenticate' header not found".into()),
        }
    }

    pub async fn exchange(
        &mut self,
        code: &str,
        login: String,
        password: String,
        device_id: String,
        url: String,
    ) -> Result<String, Box<dyn Error>> {
        if self.encrypted_code.is_empty() {
            return Err(
                "No encrypted code found. Did you call `request_code` before exchanging certs?"
                    .into(),
            );
        }
        let mut payload = payload::get_create_cert(login, password, device_id, self);
        payload.insert("code".to_owned(), code.to_string());
        payload.insert("encrypted-code".to_owned(), self.encrypted_code.clone());
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
                error!("Error to send request POST error: {err}");
                return Err("Error to send request POST".into());
            }
        };

        let data: ResCertificate = match response.json().await {
            Ok(result) => result,
            Err(error) => {
                return Err(error.to_string().into());
            }
        };

        let cert1 = match self.parse_cert(&data.certificate) {
            Ok(result) => result,
            Err(_error) => {
                return Err("Error during parse certificate 1".into());
            }
        };

        let cert2 = match self.parse_cert(&data.certificate_crypto) {
            Ok(result) => result,
            Err(_error) => {
                return Err("Error during parse certificate 2".into());
            }
        };

        let pkcs12_1 = match self.gen_cert(&self.key1, &cert1) {
            Ok(result) => result,
            Err(_error) => {
                return Err("Error during generate pkcs12 1".into());
            }
        };

        let pkcs12_2 = match self.gen_cert(&self.key2, &cert2) {
            Ok(result) => result,
            Err(_error) => {
                return Err("Error during generate pkcs12 2".into());
            }
        };

        match pkcs12_1.to_der() {
            Ok(der_data) => {
                self.cert1 = Some(der_data);
            }
            Err(_error) => {
                return Err("Error to convert PKCS12 1 to DER".into());
            }
        }
        match pkcs12_2.to_der() {
            Ok(der_data) => {
                self.cert2 = Some(der_data);
            }
            Err(_error) => {
                return Err("Error to convert PKCS12 2 to DER".into());
            }
        }
        let res = String::from("Certificates successfully exchanged!");
        Ok(res)
    }

    fn parse_cert(&self, content: &str) -> Result<X509, OpenSSLStackError> {
        let cert = X509::from_pem(content.as_bytes())?;
        Ok(cert)
    }

    fn gen_cert(&self, key: &PKey<Private>, cert: &X509) -> Result<Pkcs12, OpenSSLStackError> {
        let mut builder = Pkcs12::builder();
        builder.name("password");
        builder.pkey(key);
        builder.cert(cert);
        let p12 = builder.build2("NuRust")?;

        Ok(p12)
    }

    fn generate_key() -> Result<PKey<Private>, Box<dyn std::error::Error>> {
        let rsa = Rsa::generate(2048)?;
        let private_key = PKey::from_rsa(rsa)?;

        Ok(private_key)
    }
    pub fn get_public_key(
        private_key: &PKey<Private>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let rsa = private_key.rsa()?;
        let public_key = PKey::from_rsa(rsa)?;
        let public_key_pem = public_key.public_key_to_pem()?;
        let public_key_str = String::from_utf8(public_key_pem)?;

        Ok(public_key_str)
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
}
impl Default for Certificate {
    fn default() -> Self {
        Certificate::new()
    }
}

fn serialize_private_key<S>(key: &PKey<Private>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let pem = key.private_key_to_pem_pkcs8().unwrap();
    let pem_str = String::from_utf8_lossy(&pem);
    serializer.serialize_str(&pem_str)
}

fn deserialize_private_key<'de, D>(deserializer: D) -> Result<PKey<Private>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let pem_str: String = serde::Deserialize::deserialize(deserializer)?;
    let pem = pem_str.as_bytes();
    let key = PKey::private_key_from_pem(pem).unwrap();
    Ok(key)
}
