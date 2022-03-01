// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Usage of `match expr { x => { … } }` in these macros is intentional because it affects the
// lifetimes of temporaries. For example, we can't create a `let args = format_args!($($arg)*)`
// binding. Its usage here is similar to an absent `let x = expr in { … }` construct.
// See https://stackoverflow.com/a/48732525/1063961

use core::fmt;
use core::fmt::Write;

use crate::arch::interrupts;
use crate::console::CONSOLE;
use crate::serial::SERIAL_LOCK;

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        $crate::printk::do_println(&format_args!($($arg)*), false, false);
    });
}

#[macro_export]
macro_rules! println_force {
    ($($arg:tt)*) => ({
        $crate::printk::do_println(&format_args!($($arg)*), true, false);
    });
}

#[macro_export]
macro_rules! println_serial {
    ($($arg:tt)*) => ({
        $crate::printk::do_println(&format_args!($($arg)*), false, true);
    });
}

pub fn do_println(args: &fmt::Arguments, force: bool, no_cons: bool) {
    interrupts::disable();

    if no_cons {
        let mut serial = SERIAL_LOCK.guard();
        writeln!(&mut serial, "{}", &args).unwrap();
    } else {
        let (mut serial, mut cons_cell) = if force {
            (SERIAL_LOCK.force_unlock(), CONSOLE.force_unlock())
        } else {
            (SERIAL_LOCK.guard(), CONSOLE.guard())
        };

        let console = cons_cell.get_mut().unwrap();

        writeln!(&mut serial, "{}", &args).unwrap();
        writeln!(console, "{}", args).unwrap();
    }

    interrupts::enable();
}

// Copied from std
#[macro_export]
macro_rules! dbg {
    () => {
        $crate::println!("[{}:{}]", file!(), line!())
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::println!("[{}:{}] {} = {:#?}", file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}
