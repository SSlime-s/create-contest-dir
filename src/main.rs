mod get_request;
mod parser;
mod templates;
mod utils;

use std::{
    fmt,
    fs::{self},
    io::Write,
    process::Command,
};

use crate::{
    parser::parse_default_arg,
    templates::{CHILD_FILE_TEMPLATE, MAIN_FILE_TEMPLATE},
    utils::generate_options_file,
};

pub enum ErrorMessages {
    FailedCreateDir,
    FailedCreateFile,
    FailedWrite,
    FailedGet,
}
impl ErrorMessages {
    fn value(&self) -> &'static str {
        match *self {
            ErrorMessages::FailedCreateDir => "failed to create dir",
            ErrorMessages::FailedCreateFile => "failed to create file",
            ErrorMessages::FailedWrite => "failed to write",
            ErrorMessages::FailedGet => "failed to get file",
        }
    }
}
impl fmt::Debug for ErrorMessages {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.value())
    }
}

pub struct ContestInfo {
    name: String,
    kind: Contests,
    url: Option<String>,
}

// memo: -abc125 4問 abc126-abc211 6問 abc212- 8問
#[allow(non_camel_case_types)]
enum Contests {
    ABC,
    H_ABC,
    S_ABC,
    ARC,
    AGC,
}
impl Contests {
    fn value(&self) -> &'static str {
        match *self {
            Contests::ABC => "abc",
            Contests::H_ABC => "abc",
            Contests::S_ABC => "abc",
            Contests::ARC => "arc",
            Contests::AGC => "agc",
        }
    }

    fn problem_names(&self) -> Vec<String> {
        let a_to_d = ('a'..='d').map(|x| x.to_string()).collect::<Vec<String>>();
        let a_to_f = ('a'..='f').map(|x| x.to_string()).collect::<Vec<String>>();
        let a_to_h = ('a'..='h').map(|x| x.to_string()).collect::<Vec<String>>();
        match *self {
            Contests::ABC => a_to_h,
            Contests::H_ABC => a_to_f,
            Contests::S_ABC => a_to_d,
            Contests::ARC => a_to_f,
            Contests::AGC => a_to_f,
        }
    }
}
impl From<(&str, &str)> for Contests {
    fn from((kind, num): (&str, &str)) -> Self {
        assert!(kind == "abc" || kind == "arc" || kind == "agc");
        let num: u32 = num.parse().unwrap();
        match kind {
            "abc" => {
                if num <= 125 {
                    Contests::S_ABC
                } else if num <= 211 {
                    Contests::H_ABC
                } else {
                    Contests::ABC
                }
            }
            "arc" => Contests::ARC,
            "agc" => Contests::AGC,
            _ => Contests::AGC,
        }
    }
}

#[tokio::main]
async fn main() {
    let contest_info = match parse_default_arg() {
        Ok(c) => c,
        Err(e) => panic!("{}", e),
    };

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
