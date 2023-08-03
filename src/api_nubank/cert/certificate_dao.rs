use super::certificate::Certificate;
use crate::DB;
pub async fn create(certificate: Certificate, login: String) {
    let _created: Certificate = DB
        .create(("certificate", login.clone()))
        .content(certificate)
        .await
        .unwrap_or_default();
}
pub async fn update(certificate: Certificate, login: String) {
    let _created: Certificate = DB
        .update(("certificate", login.clone()))
        .content(certificate)
        .await
        .unwrap_or_default();
}
pub async fn get_by_login(login: String) -> Certificate {
    let select: Certificate = DB.select(("certificate", login)).await.unwrap_or_default();
    select
}
