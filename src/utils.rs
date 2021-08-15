use itertools::Itertools;
use regex::Regex;
use std::{
    fs::{File, OpenOptions},
    future::Future,
    io::{Error, Read, Seek, SeekFrom, Write},
    ops::Add,
};

use crate::{
    get_request,
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
    let base = match fetch_fn().await {
        Ok(s) => s,
        Err(_e) => return Err(ErrorMessages::FailedGet),
    };
    let mut file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(format!("{}/{}", dir_name, file_name))
    {
        Ok(f) => f,
        Err(_e) => return Err(ErrorMessages::FailedCreateFile),
    };
    match file.write_all(base.as_bytes()) {
        Ok(_) => Ok(()),
        Err(_e) => Err(ErrorMessages::FailedWrite),
    }
}

pub async fn generate_options_file(
    dir_name: &str,
    names: Vec<String>,
) -> Result<(), ErrorMessages> {
    let cargo_toml_base = match get_request::get_cargo_toml().await {
        Ok(s) => s,
        Err(_e) => return Err(ErrorMessages::FailedGet),
    };
    let re = Regex::new(r"\[dependencies\](?s:.)*").unwrap();
    let parsed_base = &re.captures(cargo_toml_base.as_str()).unwrap()[0];

    let mut cargo_toml = match OpenOptions::new()
        .read(true)
        .write(true)
        .open(format!("{}/Cargo.toml", dir_name))
    {
        Ok(f) => f,
        Err(_e) => return Err(ErrorMessages::FailedCreateFile),
    };
    let content = match clear_file(&mut cargo_toml) {
        Ok(s) => s,
        Err(_e) => return Err(ErrorMessages::FailedWrite),
    };

    match cargo_toml.write_all(
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
    ) {
        Ok(_) => (),
        Err(_e) => return Err(ErrorMessages::FailedWrite),
    };

    match std::fs::create_dir(format!("{}/.cargo", dir_name)) {
        Ok(_) => (),
        Err(_e) => return Err(ErrorMessages::FailedCreateDir),
    }
    let mut config_file = match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(format!("{}/.cargo/config.toml", dir_name))
    {
        Ok(f) => f,
        Err(_e) => return Err(ErrorMessages::FailedCreateFile),
    };
    match config_file.write_all(
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
    ) {
        Ok(_) => (),
        Err(_e) => return Err(ErrorMessages::FailedWrite),
    };

    match fetch_file(dir_name, "Cargo.lock", get_request::get_cargo_lock).await {
        Ok(_) => (),
        Err(e) => return Err(e),
    }
    match fetch_file(dir_name, "rust-toolchain", get_request::get_rust_toolchain).await {
        Ok(_) => (),
        Err(e) => return Err(e),
    }
    Ok(())
}
