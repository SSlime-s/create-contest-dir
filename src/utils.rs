mod fetch_files;
mod templates;

use itertools::Itertools;
use once_cell::sync::Lazy;
use std::{
    fs::{File, OpenOptions},
    io::{Error, Read, Seek, SeekFrom, Write},
    ops::Add,
};

use crate::{
    utils::templates::{
        CARGO_CONFIG_ALIAS_TEMPLATE, CARGO_FILE_ADD_TEMPLATE, CARGO_TOML_BIN_TEMPLATE,
    },
    ErrorMessages,
};

use self::templates::VSCODE_SETTING_TEMPLATE;

pub fn clear_file(file: &mut File) -> Result<String, Error> {
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    Ok(content)
}

async fn fetch_file<F>(dir_name: &str, file_name: &str, fetch_fn: F) -> Result<(), ErrorMessages>
where
    F: Fn() -> String,
{
    let base = fetch_fn();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(format!("{}/{}", dir_name, file_name))
        .map_err(|_e| ErrorMessages::FailedCreateFile)?;
    file.write_all(base.as_bytes())
        .map_err(|_e| ErrorMessages::FailedWrite)
}

static AFTER_DEPENDENCIES: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"\[dependencies\](?s:.)*").unwrap());
pub async fn generate_options_file(
    dir_name: &str,
    names: Vec<String>,
) -> Result<(), ErrorMessages> {
    let cargo_toml_base = fetch_files::get_cargo_toml();

    let mut cargo_toml = OpenOptions::new()
        .read(true)
        .write(true)
        .open(format!("{}/Cargo.toml", dir_name))
        .map_err(|_| ErrorMessages::FailedCreateFile)?;
    let content = clear_file(&mut cargo_toml).map_err(|_e| ErrorMessages::FailedWrite)?;

    cargo_toml
        .write_all(generate_cargo_toml_content(cargo_toml_base, content, &names).as_bytes())
        .map_err(|_e| ErrorMessages::FailedWrite)?;

    { /* generate .cargo/config.toml */
        std::fs::create_dir(format!("{}/.cargo", dir_name))
            .map_err(|_e| ErrorMessages::FailedCreateFile)?;
        let mut config_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(format!("{}/.cargo/config.toml", dir_name))
            .map_err(|_e| ErrorMessages::FailedCreateFile)?;
        config_file
            .write_all(generate_alias_content(&names).as_bytes())
            .map_err(|_e| ErrorMessages::FailedWrite)?;
    }
    { /* generate .vscode/settings.json */
        std::fs::create_dir(format!("{}/.vscode", dir_name))
            .map_err(|_e| ErrorMessages::FailedCreateFile)?;
        let mut vscode_settings_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(format!("{}/.vscode/settings.json", dir_name))
            .map_err(|_e| ErrorMessages::FailedCreateFile)?;
        vscode_settings_file
            .write_all(VSCODE_SETTING_TEMPLATE.trim_start().as_bytes())
            .map_err(|_e| ErrorMessages::FailedWrite)?;
    }

    fetch_file(dir_name, "Cargo.lock", fetch_files::get_cargo_lock).await?;
    fetch_file(dir_name, "rust-toolchain", fetch_files::get_rust_toolchain).await?;
    Ok(())
}

fn generate_alias_content(names: &[String]) -> String {
    "[alias]\n".to_string().add(
        names
            .iter()
            .map(|x| {
                CARGO_CONFIG_ALIAS_TEMPLATE
                    .trim_start()
                    .replace("{{name}}", x.as_str())
            })
            .join("\n")
            .as_str(),
    )
}

fn generate_cargo_toml_content(
    cargo_toml_base: String,
    content: String,
    names: &[String],
) -> String {
    let parsed_base = &AFTER_DEPENDENCIES
        .captures(cargo_toml_base.as_str())
        .unwrap()[0];
    content
        .trim_start()
        .trim_end()
        .replace("[dependencies]", "")
        .add(
            names
                .iter()
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
}

pub struct ProblemNames {
    idx: usize,
    now: String,
}
impl ProblemNames {
    pub fn new() -> Self {
        ProblemNames {
            idx: 0,
            now: String::new(),
        }
    }
}
impl Iterator for ProblemNames {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        let mut chars = self.now.chars().rev();
        let mut nxt = Vec::<char>::new();
        let mut done_update = false;
        loop {
            if let Some(x) = chars.next() {
                if done_update {
                    nxt.push(x);
                    continue;
                }
                match x {
                    now @ 'a'..='y' => {
                        nxt.push((now as u8 + 1) as char);
                        done_update = true;
                    }
                    'z' => nxt.push('a'),
                    _ => panic!("self.now has non-alphabetic characters !"),
                }
            } else if !done_update {
                nxt.push('a');
                done_update = true;
            } else {
                break;
            }
        }
        self.now = nxt.iter().rev().collect::<String>();
        self.idx += 1;
        Some(self.now.clone())
    }
}
