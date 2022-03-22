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

pub const CARGO_CONFIG_ALIAS_TEMPLATE: &str = r###"
run-{{name}} = "run --bin {{name}}"
{{name}} = "run-{{name}}"
test-{{name}} = "test --test {{name}}"
"###;

pub const VSCODE_SETTING_TEMPLATE: &str = r###"
{
  "rust-analyzer.server.extraEnv": {
    "RUSTUP_TOOLCHAIN": "stable"
  },
}
"###;
