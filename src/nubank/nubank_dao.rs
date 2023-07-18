use crate::{ DB};

use super::nubank::Nubank;
pub async fn create(nu: Nubank) {
    let _created: Nubank = DB
        .create(("nubank", nu.login.clone()))
        .content(nu)
        .await
        .unwrap();
}
pub async fn update(nu: Nubank) {
    let _created: Nubank = DB
        .update(("nubank", nu.login.clone()))
        .content(nu)
        .await
        .unwrap();
}
pub async fn get_by_login(login: String) -> Nubank {
    let select: Nubank = DB.select(("nubank", login)).await.unwrap();
    select
}
