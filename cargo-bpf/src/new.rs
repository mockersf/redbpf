use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use crate::CommandError;

pub fn new(path: &PathBuf, name: Option<&str>) -> Result<(), CommandError> {
    if path.exists() {
        return Err(CommandError(format!(
            "destination `{}' already exists",
            path.to_str().unwrap()
        )));
    }

    fs::create_dir_all(path.join("src"))?;
    let name = name.or_else(|| path.file_name()?.to_str()).unwrap();
    let mut file = File::create(path.join("Cargo.toml"))?;
    write!(
        &mut file,
        r#"[package]
name = "{}"
version = "0.1.0"
edition = '2018'

[dependencies]
cty = "0.2"
redbpf-macros = "0.9"
redbpf-probes = "0.9"

[build-dependencies]
bindgen = "0.51"
redbpf = {{ version = "^0.9.0", features = ["build"] }}

[features]
default = []
probes = []

[lib]
path = "src/lib.rs"
"#,
        name
    )?;

    let mut file = File::create(path.join("src").join("lib.rs"))?;
    write!(
        &mut file,
        r#"
#![feature(const_fn, const_transmute)]
#![no_std]
"#
    )?;
    Ok(())
}
