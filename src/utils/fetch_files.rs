macro_rules! atcoder_rust_base {
    ($file_name:literal) => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            concat!("/atcoder-rust-base/", $file_name)
        )
    };
}

pub async fn get_cargo_toml() -> Result<String, reqwest::Error> {
    let cargo_toml = include_str!(atcoder_rust_base!("Cargo.toml"));
    Ok(cargo_toml.to_string())
}
pub async fn get_cargo_lock() -> Result<String, reqwest::Error> {
    let cargo_lock = include_str!(atcoder_rust_base!("Cargo.lock"));
    Ok(cargo_lock.to_string())
}
pub async fn get_rust_toolchain() -> Result<String, reqwest::Error> {
    let rust_toolchain = include_str!(atcoder_rust_base!("rust-toolchain"));
    Ok(rust_toolchain.to_string())
}
