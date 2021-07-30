mod get_request;
mod templates;
mod utils;

use std::{
    fmt,
    fs::{self},
    io::Write,
    process::Command,
};

use crate::{
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

#[allow(dead_code)]
enum Contests {
    ABC,
    ARC,
    AGC,
}
#[allow(dead_code)]
impl Contests {
    fn value(&self) -> &'static str {
        match *self {
            Contests::ABC => "abc",
            Contests::ARC => "arc",
            Contests::AGC => "agc",
        }
    }

    fn problem_names(&self) -> Vec<String> {
        let a_to_f = ('a'..='f').map(|x| x.to_string()).collect::<Vec<String>>();
        let a_to_h = ('a'..='h').map(|x| x.to_string()).collect::<Vec<String>>();
        match *self {
            Contests::ABC => a_to_h,
            Contests::ARC => a_to_f,
            Contests::AGC => a_to_f,
        }
    }
}

#[tokio::main]
async fn main() {
    let contest_name = "abc-210";
    let contest_type = Contests::ABC;

    Command::new("cargo")
        .args(&["new", "--bin", contest_name])
        .output()
        .expect(ErrorMessages::FailedCreateDir.value());
    let mut main_file = fs::File::create(format!("{}/src/main.rs", contest_name))
        .expect(ErrorMessages::FailedCreateFile.value());
    main_file
        .write_all(
            MAIN_FILE_TEMPLATE
                .replace(
                    "{{mods}}",
                    contest_type
                        .problem_names()
                        .into_iter()
                        .map(|x| format!("mod {};", x))
                        .collect::<Vec<String>>()
                        .join("\n")
                        .as_str(),
                )
                .replace(
                    "{{programs}}",
                    contest_type.problem_names().join("").as_str(),
                )
                .replace(
                    "{{mains}}",
                    contest_type
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

    for x in contest_type.problem_names() {
        let mut child_file = fs::File::create(format!("{}/src/{}.rs", contest_name, x))
            .expect("failed to create file");
        child_file
            .write_all(CHILD_FILE_TEMPLATE.trim_start().as_bytes())
            .expect(ErrorMessages::FailedWrite.value());
    }

    generate_options_file(contest_name)
        .await
        .expect("Error on `generate_options_file`: ");
}
