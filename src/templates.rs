pub const CARGO_FILE_ADD_TEMPLATE: &str = r###"
# ---------------------------------------------------------------------

[dev-dependencies]
cli_test_dir = "0.1"
"###;

pub const CARGO_TOML_BIN_TEMPLATE: &str = r###"
[[bin]]
name = "{{name}}"
path = "src/{{name}}.rs"
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
