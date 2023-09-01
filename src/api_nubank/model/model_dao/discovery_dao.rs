pub async fn save_ghost_flame(ghost_flame_url: String) {
    let _update: Url = DB
        .update(("url", "ghost_flame_url"))
        .content(Url {
            link: ghost_flame_url,
        })
        .await
        .unwrap();
}
pub async fn get_ghost_flame() -> core::result::Result<String, Box<dyn Error>> {
    let ghost_flame: Url = DB.select(("url", "ghost_flame_url")).await?;
    Ok(ghost_flame.link)
}
