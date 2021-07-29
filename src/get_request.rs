use reqwest::Error;

async fn get_text(url: &str) -> Result<String, Error> {
    let s = reqwest::get(url).await?.text().await?;
    Ok(s)
}

pub async fn get_cargo_toml() -> Result<String, Error> {
    get_text("https://raw.githubusercontent.com/rust-lang-ja/atcoder-rust-base/ja-all-enabled/Cargo.toml").await
}

pub async fn get_cargo_lock() -> Result<String, Error> {
    get_text("https://raw.githubusercontent.com/rust-lang-ja/atcoder-rust-base/ja-all-enabled/Cargo.lock").await
}

pub async fn get_rust_toolchain() -> Result<String, Error> {
    get_text("https://raw.githubusercontent.com/rust-lang-ja/atcoder-rust-base/ja-all-enabled/rust-toolchain").await
}
