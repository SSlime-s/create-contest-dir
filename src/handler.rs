use itertools::Itertools;
use reqwest::{
    header::{HeaderMap, HeaderValue, COOKIE},
    Client,
};
use std::{
    fs,
    io::{BufRead, Write},
    ops::Add,
    process::Command,
};

use crate::{
    parser::extract_name_from_url,
    templates::{CHILD_FILE_TEMPLATE, TEST_FILE_CHILD_TEMPLATE, TEST_FILE_TEMPLATE},
    utils::generate_options_file,
    ContestInfo, ErrorMessages,
};

pub async fn create_contest_dir(contest_info: ContestInfo) {
    if std::path::Path::new(&format!("./{}", &contest_info.name)).is_dir() {
        panic!("Dir {} is Already Exists !", &contest_info.name)
    }
    Command::new("cargo")
        .args(&["new", "--bin", &contest_info.name, "--vcs", "none"])
        .output()
        .expect(ErrorMessages::FailedCreateDir.value());
    fs::remove_file(format!("{}/src/main.rs", &contest_info.name))
        .expect(ErrorMessages::FailedRemoveFile.value());

    for x in contest_info.kind.problem_names() {
        let mut child_file = fs::File::create(format!("{}/src/{}.rs", contest_info.name, x))
            .expect(ErrorMessages::FailedCreateFile.value());
        child_file
            .write_all(CHILD_FILE_TEMPLATE.trim_start().as_bytes())
            .expect(ErrorMessages::FailedWrite.value())
    }

    generate_options_file(&contest_info.name, contest_info.kind.problem_names())
        .await
        .expect("Error on `generate_options_file`: ");
    if let Some(_) = contest_info.url {
        generate_tests_dir(contest_info)
            .await
            .expect("Failed to Generate Tests Dir: ");
    }
}

pub async fn login(user_name: String, password: String) {
    let client = create_cli();
    let login_url = "https://atcoder.jp/login";
    let resp = client
        .get(login_url)
        .send()
        .await
        .expect("Failed to Get Login Page");
    let mut cookie_headers = HeaderMap::new();
    resp.cookies().for_each(|cookie| {
        cookie_headers.insert(
            COOKIE,
            HeaderValue::from_str(&format!("{}={}", cookie.name(), cookie.value())).unwrap(),
        );
    });
    let html = resp.text().await.expect("Failed to Get Login Page");
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
        .expect("Failed to Post Login");

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

pub async fn add_test(url: String, problem_names: Vec<String>) {
    if !std::path::Path::new("Cargo.toml").is_file() {
        panic!("Missing Cargo.toml on This Dir")
    }
    if std::path::Path::new("tests").is_dir() {
        fs::remove_dir_all("tests").expect(ErrorMessages::FailedRemoveDir.value());
    }
    fs::create_dir("tests").expect(ErrorMessages::FailedCreateDir.value());
    match generate_tests_files("tests", url, problem_names).await {
        Ok(_) => (),
        Err(e) => panic!("{}", e),
    };
}

async fn generate_tests_dir(contest_info: ContestInfo) -> Result<(), String> {
    let res = fs::create_dir(format!("{}/tests", contest_info.name));
    match res {
        Ok(_) => (),
        Err(_e) => return Err(ErrorMessages::FailedCreateDir.value().to_string()),
    }
    generate_tests_files(
        format!("{}/tests", contest_info.name),
        contest_info.url.unwrap(),
        contest_info.kind.problem_names(),
    )
    .await?;
    Ok(())
}

/*
 example:
   generate_tests_files(
       /* path: */ "abc-000/tests",
       /*  url: */ "https://atcoder.jp/contests/abc000",
       /* kind: */ Contests::ABC
   )
*/
async fn generate_tests_files(
    path: impl Into<String>,
    url: impl Into<String>,
    problem_names: Vec<String>,
) -> Result<(), String> {
    let cookie_headers = get_local_cookie_header().unwrap_or(HeaderMap::new());
    let path: String = path.into();
    let url: String = url.into();

    let client = create_cli();
    let name = match extract_name_from_url(&url) {
        Ok(s) => s,
        Err(_e) => return Err("Failed to Parse Url".to_string()),
    };
    for idx in problem_names {
        match fs::create_dir(format!("{}/{}", &path, idx)) {
            Ok(_) => (),
            Err(_e) => return Err(ErrorMessages::FailedCreateDir.value().to_string()),
        }
        let res = generate_sample_test_file(
            format!("{}/tasks/{}_{}", &url, name, idx).as_str(),
            &format!("{}/{}/{}", &path, idx, idx),
            &cookie_headers,
            &client,
        )
        .await;
        let sample_cnt = match res {
            Ok(u) => u,
            Err(_e) => return Err("Failed to Create Sample Files".to_string()),
        };
        let mut test_file = match fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(format!("{}/{}.rs", &path, idx))
        {
            Ok(f) => f,
            Err(_e) => return Err(ErrorMessages::FailedCreateFile.value().to_string()),
        };
        let res = test_file.write_all(
            TEST_FILE_TEMPLATE
                .to_string()
                .add(
                    (0..sample_cnt)
                        .map(|i| TEST_FILE_CHILD_TEMPLATE.replace("{{num}}", &(i + 1).to_string()))
                        .join("\n")
                        .as_str(),
                )
                .replace("{{name}}", &idx)
                .as_bytes(),
        );
        match res {
            Ok(_) => (),
            Err(_e) => return Err(ErrorMessages::FailedWrite.value().to_string()),
        }
    }

    Ok(())
}

/**
 example:
   generate_sample_tests_file(
       /*  url: */ "https://atcoder.jp/contests/abc000/tasks/abc000_a",
       /* path: */ "abc-000/tests/a/a",
       ...
   )
*/
async fn generate_sample_test_file(
    url: &str,
    path: &str,
    cookie_headers: &HeaderMap,
    client: &Client,
) -> Result<usize, ()> {
    let samples = fetch_sample_data(url, cookie_headers, client).await?;
    let sample_cnt = samples.len();
    for (idx, (input, output)) in samples.into_iter().enumerate() {
        let input_file = std::fs::OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(format!("{}_{}.input", path, idx + 1));
        let res = match input_file {
            Ok(mut f) => f.write_all(input.as_bytes()),
            Err(_e) => return Err(()),
        };
        if let Err(_e) = res {
            return Err(());
        }

        let output_file = std::fs::OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(format!("{}_{}.output", path, idx + 1));
        let res = match output_file {
            Ok(mut f) => f.write_all(output.as_bytes()),
            Err(_e) => return Err(()),
        };
        if let Err(_e) = res {
            return Err(());
        }
    }
    Ok(sample_cnt)
}

async fn fetch_sample_data(
    url: &str,
    cookie_headers: &HeaderMap,
    client: &Client,
) -> Result<Vec<(String, String)>, ()> {
    let resp = client.get(url).headers(cookie_headers.clone()).send().await;
    let resp = match resp {
        Ok(r) => r,
        Err(_e) => return Err(()),
    };
    let html = resp.text().await;
    let html = match html {
        Ok(s) => s,
        Err(_e) => return Err(()),
    };
    let doc = scraper::Html::parse_document(&html);

    Ok(extract_sample_data(doc))
}

fn extract_sample_data(doc: scraper::Html) -> Vec<(String, String)> {
    let expected_strings = ["入力例", "出力例"];
    let task_statement_selector = scraper::Selector::parse(r#"div[id="task-statement"]"#).unwrap();
    let pre_selector = scraper::Selector::parse("pre").unwrap();

    let mut samples = Vec::new();
    if let Some(task_statement) = doc.select(&task_statement_selector).next() {
        for pre in task_statement.select(&pre_selector) {
            if let Some(h3) = pre.prev_sibling() {
                if let Some(element) = h3.value().as_element() {
                    if element.name() == "h3" && h3.has_children() {
                        let child = h3.children().next().unwrap().value();
                        if let Some(t) = child.as_text() {
                            if expected_strings.iter().any(|&x| t.contains(x)) {
                                {
                                    samples.push(pre.text().collect::<String>());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    (0..samples.len() / 2)
        .map(|i| (samples[i * 2].clone(), samples[i * 2 + 1].clone()))
        .collect::<Vec<(String, String)>>()
}

fn create_cli() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

fn get_local_cookie_header() -> Option<HeaderMap> {
    let file = std::fs::File::open(
        dirs::home_dir()
            .unwrap()
            .join(".atcoder-create-contest-dir")
            .join("cookie"),
    );
    let file = match file {
        Ok(f) => f,
        Err(_e) => return None,
    };
    let reader = std::io::BufReader::new(file);
    let mut cookie_headers = HeaderMap::new();
    reader.lines().for_each(|line| {
        cookie_headers.insert(
            COOKIE,
            HeaderValue::from_str(&format!("{}", line.unwrap())).unwrap(),
        );
    });
    Some(cookie_headers)
}
