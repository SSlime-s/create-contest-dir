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
