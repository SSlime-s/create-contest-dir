mod get_request;
mod handler;
mod parser;
mod templates;
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
    fn value(&self) -> &'static str {
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

// memo: -abc125 4問 abc126-abc211 6問 abc212- 8問
#[allow(non_camel_case_types)]
enum Contests {
    ABC,
    H_ABC,
    S_ABC,
    ARC,
    AGC,
}
#[allow(dead_code)]
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
    let parsed_arg = match parse_arg() {
        Ok(res) => res,
        Err(e) => panic!("{}", e),
    };

    match parsed_arg {
        ParsedArg::CreateDir(contest_info) => create_contest_dir(contest_info).await,
        ParsedArg::Login(user_name, password) => login(user_name, password).await,
        ParsedArg::AddTest(url, problem_names) => add_test(url, problem_names).await,
    }
}
