use itertools::Itertools;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use std::{
    fs,
    io::{BufRead, Write},
    process::Command,
};

use crate::{
    parser::extract_name_from_url,
    templates::{CHILD_FILE_TEMPLATE, MAIN_FILE_TEMPLATE},
    utils::generate_options_file,
    ContestInfo, ErrorMessages,
};

pub async fn create_contest_dir(contest_info: ContestInfo) {
    Command::new("cargo")
        .args(&["new", "--bin", &contest_info.name])
        .output()
        .expect(ErrorMessages::FailedCreateDir.value());
    let mut main_file = fs::File::create(format!("{}/src/main.rs", contest_info.name))
        .expect(ErrorMessages::FailedCreateFile.value());
    main_file
        .write_all(
            MAIN_FILE_TEMPLATE
                .replace(
                    "{{mods}}",
                    contest_info
                        .kind
                        .problem_names()
                        .into_iter()
                        .map(|x| format!("mod {};", x))
                        .collect::<Vec<String>>()
                        .join("\n")
                        .as_str(),
                )
                .replace(
                    "{{programs}}",
                    contest_info.kind.problem_names().join("").as_str(),
                )
                .replace(
                    "{{mains}}",
                    contest_info
                        .kind
                        .problem_names()
                        .into_iter()
                        .map(|x| format!("        \"{}\" => crate::{}::main(),", x, x))
                        .collect::<Vec<String>>()
                        .join("\n")
                        .as_str(),
                )
                .trim_start()
                .as_bytes(),
        )
        .expect(ErrorMessages::FailedWrite.value());

    for x in contest_info.kind.problem_names() {
        let mut child_file = fs::File::create(format!("{}/src/{}.rs", contest_info.name, x))
            .expect("failed to create file");
        child_file
            .write_all(CHILD_FILE_TEMPLATE.trim_start().as_bytes())
            .expect(ErrorMessages::FailedWrite.value());
    }

    generate_options_file(&contest_info.name)
        .await
        .expect("Error on `generate_options_file`: ");
}

pub async fn login(user_name: String, password: String) {
    let client = create_cli();
    let login_url = "https://atcoder.jp/login";
    let resp = client
        .get(login_url)
        .send()
        .await
        .expect("failed to get login page");
    let mut cookie_headers = HeaderMap::new();
    resp.cookies().for_each(|cookie| {
        cookie_headers.insert(
            COOKIE,
            HeaderValue::from_str(&format!("{}={}", cookie.name(), cookie.value())).unwrap(),
        );
    });
    let html = resp.text().await.expect("failed to get login page");
    let document = scraper::Html::parse_document(&html);

    let selector = scraper::Selector::parse(r#"input[name="csrf_token"]"#).unwrap();
    let csrf_token = document
        .select(&selector)
        .next()
        .unwrap()
        .value()
        .attr("value")
        .unwrap()
        .to_string();

    let params = {
        let mut params = std::collections::HashMap::new();
        params.insert("username", user_name);
        params.insert("password", password);
        params.insert("csrf_token", csrf_token);
        params
    };

    let resp = client
        .post(login_url)
        .headers(cookie_headers)
        .form(&params)
        .send()
        .await
        .expect("failed to post login");

    let cookies_str = resp
        .cookies()
        .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
        .join(";");
    let path = dirs::home_dir()
        .unwrap()
        .join(".atcoder-create-contest-dir");
    std::fs::create_dir_all(path.clone()).expect(ErrorMessages::FailedCreateDir.value());
    let cookie_path = path.join("cookie");
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(cookie_path.clone())
        .expect(ErrorMessages::FailedCreateFile.value())
        .write_all(cookies_str.as_bytes())
        .expect(ErrorMessages::FailedWrite.value());
    println!("Saved Your cookie in \"{}\"", cookie_path.to_str().unwrap());
}

fn create_cli() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}
