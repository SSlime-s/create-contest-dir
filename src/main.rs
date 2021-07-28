mod templates;

use std::{fs, io::Write};

use crate::templates::{CHILD_FILE_TEMPLATE, MAIN_FILE_TEMPLATE};

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

fn main() {
    fs::create_dir("abc-210").expect(ErrorMessages::FailedCreateDir.value());
    fs::create_dir("abc-210/src").expect(ErrorMessages::FailedCreateDir.value());
    let mut main_file = fs::File::create("abc-210/src/main.rs").expect(ErrorMessages::FailedCreateFile.value());
    main_file.write_all(MAIN_FILE_TEMPLATE
        .replace("{{mods}}", (0..6)
            .map(|x| (('a' as u8 + x) as char).to_string())
            .map(|x| format!("mod {};", x))
            .collect::<Vec<String>>()
            .join("\n")
            .as_str())
        .replace("{{programs}}", (0..6)
            .map(|x| ('a' as u8 + x) as char)
            .collect::<String>()
            .as_str())
        .replace("{{mains}}", (0..6)
            .map(|x| (('a' as u8 + x) as char).to_string())
            .map(|x| format!("        \"{}\" => crate::{}::main(),", x, x))
            .collect::<Vec<String>>()
            .join("\n")
            .as_str())
        .trim_start()
        .as_bytes())
        .expect(ErrorMessages::FailedWrite.value());
    for x in 'a'..='f' {
        let mut child_file = fs::File::create(format!("abc-210/src/{}.rs", x)).expect("failed to create file");
        child_file.write_all(CHILD_FILE_TEMPLATE
            .trim_start()
            .as_bytes())
            .expect(ErrorMessages::FailedWrite.value());
    }
}
