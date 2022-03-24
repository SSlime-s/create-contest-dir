macro_rules! atcoder_rust_base {
    ($file_name:literal) => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            concat!("/atcoder-rust-base/", $file_name)
        )
    };
}

pub fn get_cargo_toml() -> String {
    include_str!(atcoder_rust_base!("Cargo.toml")).to_string()
}
pub fn get_cargo_lock() -> String {
    include_str!(atcoder_rust_base!("Cargo.lock")).to_string()
}
pub fn get_rust_toolchain() -> String {
    include_str!(atcoder_rust_base!("rust-toolchain")).to_string()
}
