/*!
Cargo subcommand for working with Rust eBPF programs.

# Overview

`cargo-bpf` is part of the [`redbpf`](https://github.com/redsift/redbpf)
project. In addition to `cargo-bpf`, the `redbpf` project includes
[`redbpf-probes`](https://redsift.github.io/rust/redbpf/doc/redbpf_probes/) and
[`redbpf-macros`](https://redsift.github.io/rust/redbpf/doc/redbpf_macros/), which
provide an idiomatic Rust API to write programs that can be compiled to eBPF
bytecode and executed by the linux in-kernel eBPF virtual machine.

# Installation

To install `cargo bpf` simply run:

```
cargo install cargo-bpf
```

# Creating a new project

After installng `cargo bpf`, you can crate a new project with `cargo bpf new`:
```ìgnore
$ cargo bpf new hello-bpf
$ ls -R hello-bpf/
hello-bpf/:
Cargo.toml  src

hello-bpf/src:
lib.rs

$ cat hello-bpf/Cargo.toml
[package]
name = "hello-bpf"
version = "0.1.0"
edition = '2018'

[dependencies]
cty = "0.2"
redbpf-macros = "0.9"
redbpf-probes = "0.9"

[features]
default = []
probes = []

[lib]
path = "src/lib.rs"

$ cat hello-bpf/src/lib.rs
#![no_std]
```

As you can see `cargo bpf new` created a new crate `hello-bpf` and
automatically added `redbpf-probes` and `redbpf-macros` as dependencies. It
also created `src/lib.rs` and declared the crate as `no_std`, as eBPF
programs are run in a restricted virtual machine where `std` features are not
available.

# Adding a new eBPF program

Adding a new program is easy:

```
$ cd hello-bpf
$ cargo bpf add block_http
$ tail Cargo.toml
...
[[bin]]
name = "block_http"
path = "src/block_http/main.rs"
required-features = ["probes"]
```

As you can see, running `cargo bpf add` added a new `[bin]` target to the
crate. This new target will contain the eBPF program code.

# Building

Say that you're building an XDP program to block all traffic directed to port 80, and have therefore modified
`src/block_http/main.rs` to include the following code:

```
#![no_std]
#![no_main]
use redbpf_probes::bindings::*;
use redbpf_probes::xdp::{XdpAction, XdpContext};
use redbpf_macros::{program, xdp};

program!(0xFFFFFFFE, "GPL");

#[xdp]
pub extern "C" fn block_port_80(ctx: XdpContext) -> XdpAction {
    if let Some(transport) = ctx.transport() {
        if transport.dest() == 80 {
            return XdpAction::Drop;
        }
    }

    XdpAction::Pass
}
```

In order to build the program, you can run:

```
$ cargo bpf build block_http
```

`cargo bpf build` will produce eBPF code compatibile with the format expected
by `redbpf::Module` and will place it in
`target/release/bpf-programs/http_block.elf`.

# Loading a program during development

`cargo bpf` includes a simple `load` subcommand that can be used during
development to test that your eBPF program is loading and producing the
expected output.

Loading eBPF programs requires admin priviledges, so you'll have to run
`load` as root or with sudo:

```
$ sudo cargo bpf load -i eth0 target/release/bpf-programs/http_block.elf
```

*/
use clap::{self, crate_authors, crate_version, App, AppSettings, Arg, SubCommand};
use std::path::PathBuf;

use cargo_bpf_lib as cargo_bpf;

fn main() {
    let matches =
        App::new("cargo")
            .bin_name("cargo")
            .settings(&[
                AppSettings::ColoredHelp,
                AppSettings::ArgRequiredElseHelp,
                AppSettings::GlobalVersion,
                AppSettings::SubcommandRequiredElseHelp,
            ])
            .subcommand(
                SubCommand::with_name("bpf")
                    .version(crate_version!())
                    .author(crate_authors!("\n"))
                    .about("A cargo subcommand for developing eBPF programs")
                    .settings(&[
                        AppSettings::SubcommandRequiredElseHelp
                    ])
                    .subcommand(
                        SubCommand::with_name("new")
                            .about("Creates a new eBPF package at <PATH>")
                            .arg(Arg::with_name("name").long("name").value_name("NAME").help(
                                "Set the resulting package name, defaults to the directory name",
                            ))
                            .arg(Arg::with_name("PATH").required(true)),
                    )
                    .subcommand(
                        SubCommand::with_name("add")
                            .about("Adds a new eBPF program at src/<NAME>")
                            .arg(Arg::with_name("NAME").required(true).help(
                                "The name of the eBPF program. The code will be created under src/<NAME>",
                            ))
                    )
                    .subcommand(
                        SubCommand::with_name("bindgen")
                            .about("Generates rust bindings from C headers")
                            .arg(Arg::with_name("HEADER").required(true).help(
                                "The C header file to generate bindings for",
                            ))
                            .arg(Arg::with_name("BINDGEN_ARGS").required(false).multiple(true).help(
                                "Extra arguments passed to bindgen",
                            ))
                    )
                    .subcommand(
                        SubCommand::with_name("build")
                            .about("Compiles the eBPF programs in the package")
                            .arg(Arg::with_name("NAME").required(false).multiple(true).help(
                                "The names of the programs to compile. When no names are specified, all the programs are built",
                            ))
                    )
                    .subcommand(
                        SubCommand::with_name("load")
                            .about("Loads the specifeid eBPF program")
                            .arg(Arg::with_name("INTERFACE").value_name("INTERFACE").short("i").long("interface").help(
                                "Binds XDP programs to the given interface"
                            ))
                            .arg(Arg::with_name("PROGRAM").required(true).help(
                                "Loads the specified eBPF program and outputs all the events generated",
                            ))
                    ),
            )
            .get_matches();
    let matches = matches.subcommand_matches("bpf").unwrap();
    if let Some(m) = matches.subcommand_matches("new") {
        let path = m.value_of("PATH").map(PathBuf::from).unwrap();

        if let Err(e) = cargo_bpf::new(&path, m.value_of("NAME")) {
            clap::Error::with_description(&e.0, clap::ErrorKind::InvalidValue).exit()
        }
    }
    if let Some(m) = matches.subcommand_matches("add") {
        if let Err(e) = cargo_bpf::new_program(m.value_of("NAME").unwrap()) {
            clap::Error::with_description(&e.0, clap::ErrorKind::InvalidValue).exit()
        }
    }
    if let Some(m) = matches.subcommand_matches("bindgen") {
        let header = m.value_of("HEADER").map(PathBuf::from).unwrap();
        let extra_args = m
            .values_of("BINDGEN_ARGS")
            .map(|i| i.collect())
            .unwrap_or_else(Vec::new);
        if let Err(e) = cargo_bpf::bindgen(&header, &extra_args[..]) {
            clap::Error::with_description(&e.0, clap::ErrorKind::InvalidValue).exit()
        }
    }
    if let Some(m) = matches.subcommand_matches("build") {
        let programs = m
            .values_of("NAME")
            .map(|i| i.map(|s| String::from(s)).collect())
            .unwrap_or_else(Vec::new);
        if let Err(e) = cargo_bpf::cmd_build(programs) {
            clap::Error::with_description(&e.0, clap::ErrorKind::InvalidValue).exit()
        }
    }
    if let Some(m) = matches.subcommand_matches("load") {
        let program = m.value_of("PROGRAM").map(PathBuf::from).unwrap();
        let interface = m.value_of("INTERFACE");
        if let Err(e) = cargo_bpf::load(&program, interface) {
            clap::Error::with_description(&e.0, clap::ErrorKind::InvalidValue).exit()
        }
    }
}
