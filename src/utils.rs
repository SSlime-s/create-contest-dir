mod fetch_files;

use itertools::Itertools;
use regex::Regex;
use std::{
    fs::{File, OpenOptions},
    future::Future,
    io::{Error, Read, Seek, SeekFrom, Write},
    ops::Add,
};

use crate::{
    templates::{CARGO_CONFIG_ALIAS_TEMPLATE, CARGO_FILE_ADD_TEMPLATE, CARGO_TOML_BIN_TEMPLATE},
    ErrorMessages,
};

pub fn clear_file(file: &mut File) -> Result<String, Error> {
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    Ok(content)
}

async fn fetch_file<F, R>(dir_name: &str, file_name: &str, fetch_fn: F) -> Result<(), ErrorMessages>
where
    F: Fn() -> R,
    R: Future<Output = Result<String, reqwest::Error>>,
{
    let base = fetch_fn().await.map_err(|_e| ErrorMessages::FailedGet)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(format!("{}/{}", dir_name, file_name))
        .map_err(|_e| ErrorMessages::FailedCreateFile)?;
    file.write_all(base.as_bytes())
        .map_err(|_e| ErrorMessages::FailedWrite)
}

pub async fn generate_options_file(
    dir_name: &str,
    names: Vec<String>,
) -> Result<(), ErrorMessages> {
    let cargo_toml_base = fetch_files::get_cargo_toml()
        .await
        .map_err(|_e| ErrorMessages::FailedGet)?;
    let re = Regex::new(r"\[dependencies\](?s:.)*").unwrap();
    let parsed_base = &re.captures(cargo_toml_base.as_str()).unwrap()[0];

    let mut cargo_toml = OpenOptions::new()
        .read(true)
        .write(true)
        .open(format!("{}/Cargo.toml", dir_name))
        .map_err(|_| ErrorMessages::FailedCreateFile)?;
    let content = clear_file(&mut cargo_toml).map_err(|_e| ErrorMessages::FailedWrite)?;

    cargo_toml
        .write_all(
            content
                .trim_start()
                .trim_end()
                .replace("[dependencies]", "")
                .add(
                    (&names)
                        .into_iter()
                        .map(|x| {
                            CARGO_TOML_BIN_TEMPLATE
                                .trim()
                                .replace("{{name}}", x.as_str())
                        })
                        .join("\n")
                        .as_str(),
                )
                .add("\n\n")
                .add(parsed_base)
                .add(CARGO_FILE_ADD_TEMPLATE)
                .as_bytes(),
        )
        .map_err(|_e| ErrorMessages::FailedWrite)?;

    std::fs::create_dir(format!("{}/.cargo", dir_name))
        .map_err(|_e| ErrorMessages::FailedCreateFile)?;
    let mut config_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(format!("{}/.cargo/config.toml", dir_name))
        .map_err(|_e| ErrorMessages::FailedCreateFile)?;
    config_file
        .write_all(
            "[alias]\n"
                .to_string()
                .add(
                    names
                        .into_iter()
                        .map(|x| {
                            CARGO_CONFIG_ALIAS_TEMPLATE
                                .trim_start()
                                .replace("{{name}}", x.as_str())
                        })
                        .join("\n")
                        .as_str(),
                )
                .as_bytes(),
        )
        .map_err(|_e| ErrorMessages::FailedWrite)?;

    fetch_file(dir_name, "Cargo.lock", fetch_files::get_cargo_lock).await?;
    fetch_file(dir_name, "rust-toolchain", fetch_files::get_rust_toolchain).await?;
    Ok(())
}
