const JA_ALL_ENABLED: &'static str =
    "https://raw.githubusercontent.com/rust-lang-ja/atcoder-rust-base/ja-all-enabled";
async fn get_ja_all_enabled_text(file_path: &str) -> Result<String, reqwest::Error> {
    reqwest::get(&format!("{}/{}", JA_ALL_ENABLED, file_path))
        .await?
        .text()
        .await
}

pub async fn get_cargo_toml() -> Result<String, reqwest::Error> {
    get_ja_all_enabled_text("Cargo.toml").await
}
pub async fn get_cargo_lock() -> Result<String, reqwest::Error> {
    get_ja_all_enabled_text("Cargo.lock").await
}
pub async fn get_rust_toolchain() -> Result<String, reqwest::Error> {
    get_ja_all_enabled_text("rust-toolchain").await
}
