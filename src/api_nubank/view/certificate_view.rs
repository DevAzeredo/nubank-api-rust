use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct CertificateRequest {
    pub(crate) login: String,
    pub(crate) password: String,
}
#[derive(Debug, Serialize)]
pub(crate) struct CertificateResponse {
    pub(crate) message: String,
    pub(crate) email: String,
}
