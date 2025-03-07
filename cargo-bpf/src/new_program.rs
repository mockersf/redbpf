use std::fs::{self, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use toml_edit;

use crate::CommandError;

pub fn new_program(name: &str) -> Result<(), CommandError> {
    use toml_edit::{value, Array, ArrayOfTables, Document, Item, Table};

    let current_dir = std::env::current_dir().unwrap();
    let path = Path::new("Cargo.toml");
    if !path.exists() {
        return Err(CommandError(format!(
            "Could not find `Cargo.toml' in {:?}",
            current_dir
        )));
    }
    let data = fs::read_to_string(path).unwrap();
    let mut config = data.parse::<Document>().unwrap();

    let crate_name = config["lib"]["name"]
        .as_str()
        .or(config["package"]["name"].as_str())
        .ok_or(CommandError("invalid manifest syntax".to_string()))
        .map(String::from)?;

    let mut targets = match &config["bin"] {
        Item::None => ArrayOfTables::new(),
        Item::ArrayOfTables(array) => array.clone(),
        _ => return Err(CommandError(format!("invalid manifest syntax"))),
    };
    if targets
        .iter()
        .any(|target| target["name"].as_str().map(|s| s == name).unwrap_or(false))
    {
        return Err(CommandError(format!(
            "a program named `{}' already exists",
            name
        )));
    }

    let mut target = Table::new();
    target.entry("name").or_insert(value(name));
    target
        .entry("path")
        .or_insert(value(format!("src/{}/main.rs", name)));
    let mut features = Array::default();
    features.push("probes");
    target.entry("required-features").or_insert(value(features));

    targets.append(target);
    config["bin"] = Item::ArrayOfTables(targets);

    fs::write(path, config.to_string())?;

    let src = Path::new("src");
    let lib_rs = src.join("lib.rs");
    let mut file = OpenOptions::new().write(true).open(lib_rs)?;
    file.seek(SeekFrom::End(0))?;
    write!(&mut file, "pub mod {};\n", name)?;

    let probe_dir = src.join(name);
    fs::create_dir_all(probe_dir.clone())?;

    let mod_rs = probe_dir.join("mod.rs");
    fs::write(
        mod_rs,
        r#"
use cty::*;

// This is where you should define the types shared by the kernel and user
// space, eg:
//
// #[repr(C)]
// #[derive(Debug)]
// pub struct SomeEvent {
//     pub pid: c_ulonglong,
//     ...
// }
"#,
    )?;
    let main_rs = probe_dir.join("main.rs");
    let mut main_rs = File::create(main_rs)?;
    write!(
        &mut main_rs,
        r#"
#![no_std]
#![no_main]

use cty::*;

use redbpf_probes::bindings::*;
use redbpf_probes::maps::*;
use redbpf_macros::{{map, program, kprobe}};

// Use the types you're going to share with userspace, eg:
// use {lib}::{name}::SomeEvent;

program!(0xFFFFFFFE, "GPL");

// The maps and probe functions go here, eg:
//
// #[map("syscall_events")]
// static mut syscall_events: PerfMap<SomeEvent> = PerfMap::new();
//
// #[kprobe("syscall_enter")]
// pub extern "C" fn syscall_enter(ctx: *mut c_void) -> i32 {{
//   let pid_tgid = bpf_get_current_pid_tgid();
//   ...
//
//   let event = SomeEvent {{
//     id: pid_tgid >> 32,
//     ...
//   }};
//   unsafe {{ syscall_events.insert(ctx, event) }};
//
//   return 0;
// }}
"#,
        lib = crate_name,
        name = name
    )?;

    Ok(())
}
