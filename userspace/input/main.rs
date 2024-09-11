// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![no_main]
#![feature(format_args_nl)]

use ulib::{print, println};

#[no_mangle]
fn main() {
    loop {
        print!("Enter character: ");
        let ch = ulib::getch(false);
        println!("\nYou wrote: {ch}");

        print!("Your name: ");
        let name = ulib::readline();
        println!("\nHello, {name}!");
    }
}
