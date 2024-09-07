//usr/bin/env rustc "$0" && ./config "$@"; exit

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![feature(iter_intersperse)]
#![allow(non_camel_case_types, unused)]

macro_rules! options {
    (
        $(
            $( #[$doc:meta] )*
            $id:ident: $t:ty = $v:expr,
        )+
    ) => {
        struct Options {
            $(
                $( #[$doc] )*
                $id: $t,
            )+
        }

        const TYPECHECK: Options = Options {
            $( $id: $v, )+
        };

        const OPTIONS: &[(&str, &str)] = &[
            $( (stringify!($id), stringify!($v)), )+
        ];
    }
}

options! {
    /// Target architecture
    arch: Arch = Arch::x64,

    /// Enable graphic framebuffer console in qemu
    graphic: bool = true,

    /// Enable serial input/output
    serial: bool = true,

    /// Enable output of trace!() macro
    trace: bool = false,
}

enum Arch {
    x64,
    aarch64,
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let strs = args.iter().map(String::as_str).collect::<Vec<&str>>();

    match &strs[..] {
        [_p, "-m" | "--make"] => output_makefile(),
        [_p, "-c" | "--cargo"] => output_cargo_args(),
        _ => {
            eprintln!("Usage: ./config.rs --make | --cargo");
            std::process::exit(1);
        }
    };
}

fn output_makefile() {
    println!("# Autogenerated from config.rs");

    for (key, val) in OPTIONS {
        let leaf = val.split("::").last().unwrap();
        println!("CFG_{} = {}", key.to_uppercase(), leaf);
    }
}

fn output_cargo_args() {
    let mut flags = vec![];

    for (key, val) in OPTIONS {
        match *val {
            "false" => continue,
            "true" => {
                flags.push(format!("--cfg={}", key));
            }
            _ => {
                let leaf = val.split("::").last().unwrap();
                flags.push(format!("--cfg={}_{}", key, leaf));
            }
        }
    }

    let output = flags.into_iter().intersperse("\x1f".to_owned()).collect::<String>();

    print!("CARGO_ENCODED_RUSTFLAGS='{}'", output);
}