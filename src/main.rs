mod templates;

use std::{
    fs::{self, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    process::Command,
};

use crate::templates::{CARGO_TOML, CHILD_FILE_TEMPLATE, MAIN_FILE_TEMPLATE};

enum ErrorMessages {
    FailedCreateDir,
    FailedCreateFile,
    FailedWrite,
}
impl ErrorMessages {
    fn value(&self) -> &'static str {
        match *self {
            ErrorMessages::FailedCreateDir => "failed to create dir",
            ErrorMessages::FailedCreateFile => "failed to create file",
            ErrorMessages::FailedWrite => "failed to write",
        }
    }
}

enum Contests {
    ABC,
    ARC,
    AGC,
}
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

fn main() {
    Command::new("cargo")
        .args(&["new", "--bin", "abc-210"])
        .output()
        .expect(ErrorMessages::FailedCreateDir.value());
    let mut main_file =
        fs::File::create("abc-210/src/main.rs").expect(ErrorMessages::FailedCreateFile.value());
    let contest_name = "abc-210";
    let contest_type = Contests::ABC;
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
        let mut child_file =
            fs::File::create(format!("{}/src/{}.rs", contest_name ,x)).expect("failed to create file");
        child_file
            .write_all(CHILD_FILE_TEMPLATE.trim_start().as_bytes())
            .expect(ErrorMessages::FailedWrite.value());
    }

    let mut cargo_toml = OpenOptions::new()
        .read(true)
        .write(true)
        .open(format!("{}/Cargo.toml", contest_name))
        .expect(ErrorMessages::FailedCreateFile.value());
    let mut contents = String::new();
    cargo_toml
        .read_to_string(&mut contents)
        .expect(ErrorMessages::FailedWrite.value());
    cargo_toml
        .set_len(0)
        .expect(ErrorMessages::FailedWrite.value());
    cargo_toml
        .seek(SeekFrom::Start(0))
        .expect(ErrorMessages::FailedWrite.value());
    cargo_toml
        .write_all(
            contents
                .trim_start()
                .replace("[dependencies]", CARGO_TOML)
                .as_bytes(),
        )
        .expect(ErrorMessages::FailedWrite.value());
}
