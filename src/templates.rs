pub const CARGO_FILE_ADD_TEMPLATE: &str = r###"
# ---------------------------------------------------------------------

[dev-dependencies]
cli_test_dir = "0.1"
"###;

pub const CHILD_FILE_TEMPLATE: &str = r###"
use proconio::input;

pub fn main() {
    input! {
        n: usize,
    }
    println!("{}", n);
}
"###;

pub const MAIN_FILE_TEMPLATE: &str = r###"
{{mods}}

use std::env;

const PROGRAMS: &str = "{{programs}}";

fn is_valid_program(index: &String) -> bool {
    let trimmed_index = index.trim();
    trimmed_index.len() == 1 && PROGRAMS.chars().any(|x| x.to_string() == trimmed_index.to_lowercase())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let problem_idx = if args.len() == 2 && is_valid_program(&args[1]) {
        args[1].to_lowercase()
    } else {
        panic!("Args is Invalid! For example, 'cargo run a'");
    };
    match problem_idx.as_str() {
{{mains}}
        &_ => (),
    };
}
"###;

pub const TEST_FILE_TEMPLATE: &str = r##"
use std::io::BufRead;

use cli_test_dir::*;

const BIN: &'static str = "./{{name}}";

fn test_base(name: &str, num: u32) {
    let testdir = TestDir::new(BIN, "");
    let input = std::io::BufReader::new(std::fs::File::open(format!("tests/{}/{}_{}.input", name, name, num)).unwrap())
        .lines()
        .map(|line| line.unwrap())
        .collect::<Vec<String>>()
        .join("\n");
    let output = testdir.cmd().arg(name).output_with_stdin(input).expect_success();
    let expect_output = std::io::BufReader::new(std::fs::File::open(format!("tests/{}/{}_{}.output", name, name, num)).unwrap())
        .lines()
        .map(|line| line.unwrap())
        .collect::<Vec<String>>()
        .join("\n");
    assert_eq!(
        output.stdout_str().trim_end_matches("\n"),
        expect_output.trim_end_matches("\n")
    );
}
"##;

pub const TEST_FILE_CHILD_TEMPLATE: &str = r###"
#[test]
fn sample_{{name}}_{{num}}() {
    test_base("{{name}}", {{num}});
}
"###;
