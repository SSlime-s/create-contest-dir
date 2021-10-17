mod templates;

use itertools::Itertools;
use once_cell::sync::Lazy;
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
    handler::templates::{CHILD_FILE_TEMPLATE, TEST_FILE_CHILD_TEMPLATE, TEST_FILE_TEMPLATE},
    utils::generate_options_file,
    ContestInfo, ErrorMessages,
};

pub async fn create_contest_dir(contest_info: ContestInfo) {
    let name = contest_info.name.clone();
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
        .expect("Error on `generate_options_file`");
    if contest_info.url.is_some() {
        generate_tests_dir(contest_info)
            .await
            .expect("Failed to Generate Tests Dir");
    }
    println!("Success to Create Contest Dir on `./{}`", name);
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

    let params: std::collections::HashMap<&str, String> = [
        ("username", user_name),
        ("password", password),
        ("csrf_token", csrf_token),
    ]
    .iter()
    .cloned()
    .collect();

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
    generate_tests_files("tests", url, problem_names)
        .await
        .expect("Failed on `generate_tests_files`");
}

async fn generate_tests_dir(contest_info: ContestInfo) -> Result<(), String> {
    fs::create_dir(format!("{}/tests", contest_info.name))
        .map_err(|_e| ErrorMessages::FailedCreateDir)?;
    generate_tests_files(
        format!("{}/tests", contest_info.name),
        contest_info.url.unwrap(),
        contest_info.kind.problem_names(),
    )
    .await?;
    Ok(())
}

/**
example:
 ```
   generate_tests_files(
       /*     path: */ "abc-000/tests",
       /* base_url: */ "https://atcoder.jp/contests/abc000",
       /*     kind: */ Contests::ABC
   )
 ```
*/
async fn generate_tests_files(
    path: impl Into<String>,
    base_url: impl Into<String>,
    problem_names: Vec<String>,
) -> Result<(), String> {
    let cookie_headers = get_local_cookie_header().unwrap_or_default();
    let path: String = path.into();
    let url: String = base_url.into();

    let client = create_cli();
    let problem_urls =
        fetch_sample_urls(&format!("{}/tasks", url), &cookie_headers, &client).await?;
    for (idx, url) in problem_names.into_iter().zip(problem_urls) {
        fs::create_dir(format!("{}/{}", &path, idx))
            .map_err(|_e| ErrorMessages::FailedCreateDir)?;
        let sample_cnt = generate_sample_test_file(
            url.as_str(),
            &format!("{}/{}/{}", &path, idx, idx),
            &cookie_headers,
            &client,
        )
        .await
        .map_err(|_e| "Failed to Create Sample Files")?;
        let mut test_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(format!("{}/{}.rs", &path, idx))
            .map_err(|_e| ErrorMessages::FailedCreateFile)?;
        test_file
            .write_all(
                TEST_FILE_TEMPLATE
                    .to_string()
                    .add(
                        (0..sample_cnt)
                            .map(|i| {
                                TEST_FILE_CHILD_TEMPLATE.replace("{{num}}", &(i + 1).to_string())
                            })
                            .join("\n")
                            .as_str(),
                    )
                    .replace("{{name}}", &idx)
                    .as_bytes(),
            )
            .map_err(|_e| ErrorMessages::FailedWrite)?;
    }

    Ok(())
}

static TABLE_SELECTOR: Lazy<scraper::Selector> =
    Lazy::new(|| scraper::Selector::parse("table").unwrap());
static TH_SELECTOR: Lazy<scraper::Selector> =
    Lazy::new(|| scraper::Selector::parse("thead th").unwrap());
static TR_SELECTOR: Lazy<scraper::Selector> =
    Lazy::new(|| scraper::Selector::parse("tbody tr").unwrap());
static TD_SELECTOR: Lazy<scraper::Selector> = Lazy::new(|| scraper::Selector::parse("td").unwrap());
static A_SELECTOR: Lazy<scraper::Selector> = Lazy::new(|| scraper::Selector::parse("a").unwrap());
async fn fetch_sample_urls(
    tasks_url: &str,
    cookie_headers: &HeaderMap,
    client: &Client,
) -> Result<Vec<String>, String> {
    let html = client
        .get(tasks_url)
        .headers(cookie_headers.clone())
        .send()
        .await
        .map_err(|_e| _e.to_string())?
        .text()
        .await
        .map_err(|_e| _e.to_string())?;
    let doc = scraper::Html::parse_document(&html);

    for table in doc.select(&TABLE_SELECTOR) {
        let pos = match table
            .select(&TH_SELECTOR)
            .position(|element| match element.text().next() {
                Some(text) => text == "問題名" || text == "Task Name",
                None => false,
            }) {
            Some(p) => p,
            None => continue,
        };

        let res = table
            .select(&TR_SELECTOR)
            .map(|tr_element| {
                let td_elements = tr_element
                    .select(&TD_SELECTOR)
                    .collect::<Vec<scraper::ElementRef>>();
                let link = td_elements[pos]
                    .select(&A_SELECTOR)
                    .next()
                    .unwrap()
                    .value()
                    .attr("href")
                    .unwrap();
                "https://atcoder.jp".to_string() + link
            })
            .collect::<Vec<String>>();
        return Ok(res);
    }

    Err("EOF".into())
}

/**
example:
 ```
   generate_sample_tests_file(
       /*  url: */ "https://atcoder.jp/contests/abc000/tasks/abc000_a",
       /* path: */ "abc-000/tests/a/a",
       ...
   )
 ```
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
        // input のファイルを作って書き込む
        std::fs::OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(format!("{}_{}.input", path, idx + 1))
            .map_err(|_e| ())?
            .write_all(input.as_bytes())
            .map_err(|_e| ())?;

        // output のファイルを作って書き込む
        std::fs::OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(format!("{}_{}.output", path, idx + 1))
            .map_err(|_e| ())?
            .write_all(output.as_bytes())
            .map_err(|_e| ())?;
    }
    Ok(sample_cnt)
}

async fn fetch_sample_data(
    url: &str,
    cookie_headers: &HeaderMap,
    client: &Client,
) -> Result<Vec<(String, String)>, ()> {
    let html = client
        .get(url)
        .headers(cookie_headers.clone())
        .send()
        .await
        .map_err(|_e| ())?
        .text()
        .await
        .map_err(|_e| ())?;
    let doc = scraper::Html::parse_document(&html);

    Ok(extract_sample_data(doc))
}

static TASK_STATEMENT_SELECTOR: Lazy<scraper::Selector> =
    Lazy::new(|| scraper::Selector::parse(r#"div[id="task-statement"]"#).unwrap());
static PRE_SELECTOR: Lazy<scraper::Selector> =
    Lazy::new(|| scraper::Selector::parse("pre").unwrap());
fn extract_sample_data(doc: scraper::Html) -> Vec<(String, String)> {
    let expected_strings = ["入力例", "出力例"];

    let mut samples = Vec::new();
    if let Some(task_statement) = doc.select(&TASK_STATEMENT_SELECTOR).next() {
        for pre in task_statement.select(&PRE_SELECTOR) {
            if let Some(h3) = pre.prev_sibling() {
                if let Some(element) = h3.value().as_element() {
                    if element.name() == "h3" && h3.has_children() {
                        let child = h3.children().next().unwrap().value();
                        if let Some(t) = child.as_text() {
                            if expected_strings.iter().any(|&x| t.contains(x)) {
                                samples.push(pre.text().collect::<String>());
                            }
                        }
                    }
                }
            }
        }
    }
    samples
        .chunks_exact(2)
        .map(|v| (v.get(0).unwrap().clone(), v.get(1).unwrap().clone()))
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
    )
    .ok()?;

    let reader = std::io::BufReader::new(file);
    let mut cookie_headers = HeaderMap::new();
    reader.lines().for_each(|line| {
        cookie_headers.insert(
            COOKIE,
            HeaderValue::from_str(line.unwrap().as_str()).unwrap(),
        );
    });
    Some(cookie_headers)
}
