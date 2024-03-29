mod handler;
mod parser;
mod utils;

use std::fmt;

use crate::{
    handler::{add_test, create_contest_dir, login},
    parser::{parse_arg, ParsedArg},
};

pub enum ErrorMessages {
    FailedCreateDir,
    FailedCreateFile,
    FailedRemoveDir,
    FailedRemoveFile,
    FailedWrite,
    FailedGet,
}
impl ErrorMessages {
    fn value<'a>(&self) -> &'a str {
        match *self {
            ErrorMessages::FailedCreateDir => "Failed to Create Dir",
            ErrorMessages::FailedCreateFile => "Failed to Create File",
            ErrorMessages::FailedRemoveDir => "Failed to Remove Dir",
            ErrorMessages::FailedRemoveFile => "Failed to Remove File",
            ErrorMessages::FailedWrite => "Failed to Write",
            ErrorMessages::FailedGet => "Failed to Get File",
        }
    }
}
impl fmt::Debug for ErrorMessages {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.value())
    }
}
impl From<ErrorMessages> for String {
    fn from(msg: ErrorMessages) -> String {
        msg.value().into()
    }
}

pub struct ContestInfo {
    name: String,
    kind: Contests,
    url: Option<String>,
}

#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
enum Contests {
    ABC,
    H_ABC,
    S_ABC,
    ARC,
    AGC,
}
#[allow(dead_code)]
impl Contests {
    fn value<'a>(&self) -> &'a str {
        match *self {
            Contests::ABC => "abc",
            Contests::H_ABC => "abc",
            Contests::S_ABC => "abc",
            Contests::ARC => "arc",
            Contests::AGC => "agc",
        }
    }

    fn problem_names(&self) -> Vec<String> {
        fn create_a_to_x(n: usize) -> Vec<String> {
            crate::utils::ProblemNames::new().take(n).collect()
        }
        match *self {
            Contests::ABC => create_a_to_x(8),
            Contests::H_ABC | Contests::ARC | Contests::AGC => create_a_to_x(6),
            Contests::S_ABC => create_a_to_x(4),
        }
    }

    pub fn from_typename<T>(type_name: T) -> Option<Contests>
    where
        String: From<T>,
    {
        match String::from(type_name).as_str() {
            "abc" => Contests::ABC,
            "h-abc" => Contests::H_ABC,
            "s-abc" => Contests::S_ABC,
            "arc" => Contests::ARC,
            "agc" => Contests::AGC,
            _ => None?,
        }
        .into()
    }
}
impl From<(&str, &str)> for Contests {
    fn from((kind, num): (&str, &str)) -> Self {
        assert!(kind == "abc" || kind == "arc" || kind == "agc");
        let num: u32 = num.parse().unwrap();
        // -ABC125 4問 ABC126-ABC211 6問 abc212- 8問
        match kind {
            "abc" if num <= 125 => Contests::S_ABC,
            "abc" if num <= 211 => Contests::H_ABC,
            "abc" => Contests::ABC,
            "arc" => Contests::ARC,
            "agc" => Contests::AGC,
            _ => Contests::AGC,
        }
    }
}

#[tokio::main]
async fn main() {
    let parsed_arg = parse_arg().expect("Failed to Parse Arg");

    match parsed_arg {
        ParsedArg::CreateDir(contest_info) => create_contest_dir(contest_info).await,
        ParsedArg::Login(user_name, password) => login(user_name, password).await,
        ParsedArg::AddTest(url, problem_names) => add_test(url, problem_names).await,
    }
}
