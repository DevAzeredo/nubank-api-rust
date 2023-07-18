use std::collections::HashMap;

use crate::cert::certificate::Certificate;

pub fn get_auth_cert(login: String, password: String) -> HashMap<String, String> {
    let mut payload: HashMap<String, String> = HashMap::new();
    payload.insert("grant_type".to_string(), "password".to_string());
    payload.insert("client_id".to_string(), "legacy_client_id".to_string());
    payload.insert(
        "client_secret".to_string(),
        "legacy_client_secret".to_string(),
    );
    payload.insert("password".to_string(), password);
    payload.insert("login".to_string(), login);
    payload
}

pub fn get_create_cert(
    login: String,
    password: String,
    device_id: String,
    certificate: &mut Certificate,
) -> HashMap<String, String> {
    let mut payload = HashMap::new();
    payload.insert("login".to_string(), login.clone());
    payload.insert("password".to_string(), password.clone());
    payload.insert(
        "public_key".to_string(),
        Certificate::get_public_key(&certificate.key1).unwrap(),
    );
    payload.insert(
        "public_key_crypto".to_string(),
        Certificate::get_public_key(&certificate.key2).unwrap(),
    );
    payload.insert(
        "model".to_string(),
        format!("NuRust ({})", device_id.to_string()),
    );
    payload.insert("device_id".to_string(), device_id.to_string());
    payload
}
