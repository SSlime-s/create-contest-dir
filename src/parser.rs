use std::io::{stdin, stdout, Write};

use clap::{
    self, crate_authors, crate_description, crate_name, crate_version, App, Arg, ArgMatches,
    SubCommand,
};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{ContestInfo, Contests};

struct OptionalContestInfo {
    name: Option<String>,
    kind: Option<Contests>,
    url: Option<String>,
}
impl From<OptionalContestInfo> for Result<ContestInfo, ()> {
    fn from(info: OptionalContestInfo) -> Result<ContestInfo, ()> {
        if let (Some(name), Some(kind)) = (info.name, info.kind) {
            Ok(ContestInfo {
                name,
                kind,
                url: info.url,
            })
        } else {
            Err(())
        }
    }
}

pub enum ParsedArg {
    CreateDir(ContestInfo),
    Login(String, String),
    AddTest(String, Vec<String>),
}
pub fn parse_arg() -> Result<ParsedArg, String> {
    let app = create_app();
    let matches = app.get_matches();
    if let Some(matches) = matches.subcommand_matches("login") {
        match parse_login_arg(matches) {
            Ok(res) => Ok(ParsedArg::Login(res.0, res.1)),
            Err(e) => Err(e),
        }
    } else if let Some(matches) = matches.subcommand_matches("add_test") {
        match parse_add_test_arg(matches) {
            Ok(res) => Ok(ParsedArg::AddTest(res.0, res.1)),
            Err(e) => Err(e),
        }
    } else {
        match parse_default_arg(&matches) {
            Ok(res) => Ok(ParsedArg::CreateDir(res)),
            Err(e) => Err(e),
        }
    }
}

fn parse_login_arg(matches: &ArgMatches) -> Result<(String, String), String> {
    let user_name = if let Some(u) = matches.value_of("user_name") {
        u.to_string()
    } else {
        print!("user name: ");
        stdout().flush().unwrap();
        let mut name = String::new();
        stdin().read_line(&mut name).unwrap();
        name.trim().to_string()
    };
    let password =
        rpassword::read_password_from_tty(Some("password: ")).map_err(|e| e.to_string())?;
    Ok((user_name, password))
}

fn parse_add_test_arg(matches: &ArgMatches) -> Result<(String, Vec<String>), String> {
    let mut url = None;
    let mut kind: Option<Contests> = None;
    if let Some(v_url) = matches.value_of("url") {
        let extracted_name = extract_name_from_url(v_url);
        match extracted_name {
            Ok(v_name) => {
                url = Some(format!("https://atcoder.jp/contests/{}", v_name));
                let formatted_name = format_contest_name(&v_name);
                match formatted_name {
                    ContestKind::AXC(v_kind, v_num) => {
                        kind = Some((v_kind.as_str(), v_num.as_str()).into());
                    }
                    ContestKind::Other(_name) => (),
                }
            }
            Err(_e) => return Err("Invalid URL !".to_string()),
        }
    }

    if let Some(v_type) = matches.value_of("type") {
        kind = Some(match v_type.to_lowercase().as_str() {
            "abc" => Contests::ABC,
            "h-abc" => Contests::H_ABC,
            "s-abc" => Contests::S_ABC,
            "arc" => Contests::ARC,
            "agc" => Contests::AGC,
            _ => return Err("invalid kind !".to_string()),
        });
    }

    if let (Some(url), Some(kind)) = (url, kind) {
        Ok((url, kind.problem_names()))
    } else {
        Err("invalid args".to_string())
    }
}

fn parse_default_arg(matches: &ArgMatches) -> Result<ContestInfo, String> {
    let mut contest_info = OptionalContestInfo {
        name: None,
        kind: None,
        url: None,
    };

    if let Some(v_url) = matches.value_of("url") {
        let extracted_name = extract_name_from_url(v_url);
        match extracted_name {
            Ok(v_name) => {
                contest_info.url = Some(format!("https://atcoder.jp/contests/{}", v_name));
                let formatted_name = format_contest_name(&v_name);
                match formatted_name {
                    ContestKind::AXC(kind, num) => {
                        contest_info.name = Some(format!("{}-{}", kind, num));
                        contest_info.kind = Some((kind.as_str(), num.as_str()).into());
                    }
                    ContestKind::Other(name) => contest_info.name = Some(name),
                }
            }
            Err(_e) => return Err("Invalid URL !".to_string()),
        }
    }

    if let Some(v_name) = matches.value_of("name") {
        let formatted_name = format_contest_name(&v_name);
        match formatted_name {
            ContestKind::AXC(kind, num) => {
                contest_info.name = Some(format!("{}-{}", kind, num));
                contest_info.kind = Some((kind.as_str(), num.as_str()).into());
            }
            ContestKind::Other(name) => contest_info.name = Some(name),
        }
    }

    if let Some(v_type) = matches.value_of("type") {
        contest_info.kind = Some(match v_type.to_lowercase().as_str() {
            "abc" => Contests::ABC,
            "h-abc" => Contests::H_ABC,
            "s-abc" => Contests::S_ABC,
            "arc" => Contests::ARC,
            "agc" => Contests::AGC,
            _ => return Err("invalid kind !".to_string()),
        });
    }

    let r: Result<ContestInfo, ()> = contest_info.into();
    match r {
        Ok(c) => Ok(c),
        Err(_) => Err("name and kind is required !".to_string()),
    }
}

fn create_app<'a>() -> App<'a, 'a> {
    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("url")
                .help("contest url")
                .short("u")
                .long("url")
                .value_name("URL")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("name")
                .help("contest_name")
                .short("n")
                .long("name")
                .value_name("NAME")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("type")
                .help("contest type")
                .short("t")
                .long("type")
                .value_name("TYPE")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("login")
                .about("login to AtCoder (for contest on going)")
                .arg(
                    Arg::with_name("user_name")
                        .help("user name")
                        .short("u")
                        .long("user")
                        .value_name("USER_NAME")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("add_test")
                .about("add test on existing dir")
                .arg(
                    Arg::with_name("url")
                        .help("contest url")
                        .short("u")
                        .long("url")
                        .value_name("URL")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("type")
                        .help("contest type")
                        .short("t")
                        .long("type")
                        .value_name("TYPE")
                        .takes_value(true),
                ),
        );
    app
}

static URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^https?://atcoder.jp/contests/([^/]+).*$").unwrap());

pub fn extract_name_from_url(url: &str) -> Result<String, ()> {
    let m = URL_REGEX.captures(url);
    match m {
        Some(c) => Ok(c[1].to_string()),
        None => Err(()),
    }
}

static AXC_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^(a[bgr]c)[-_]?([0-9]{3})$").unwrap());
enum ContestKind {
    AXC(String, String),
    Other(String),
}
fn format_contest_name(name: &str) -> ContestKind {
    let m = AXC_REGEX.captures(name);
    match m {
        Some(c) => ContestKind::AXC((&c[1]).to_lowercase().to_string(), (&c[2]).to_string()),
        None => ContestKind::Other(name.to_lowercase().replace("_", "-")),
    }
}
