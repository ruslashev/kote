// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::fmt;
use core::fmt::Write;

use crate::arch::interrupts;
use crate::console::CONSOLE;
use crate::serial::SERIAL;

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        $crate::printk::do_print(&format_args!($($arg)*), true, false, false)
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::printk::do_print(&format_args!($($arg)*), false, false, false)
    }
}

#[macro_export]
macro_rules! println_force {
    ($($arg:tt)*) => {
        $crate::printk::do_print(&format_args!($($arg)*), true, true, false)
    }
}

#[macro_export]
macro_rules! println_serial {
    ($($arg:tt)*) => {
        $crate::printk::do_print(&format_args!($($arg)*), true, false, true)
    }
}

#[macro_export]
macro_rules! print_serial {
    ($($arg:tt)*) => {
        $crate::printk::do_print(&format_args!($($arg)*), false, false, true)
    }
}

#[macro_export]
macro_rules! println_serial_force {
    ($($arg:tt)*) => {
        $crate::printk::do_print(&format_args!($($arg)*), true, true, true)
    }
}

pub fn do_print(args: &fmt::Arguments, newline: bool, force: bool, no_cons: bool) {
    interrupts::with_disabled(|| {
        let mut serial = if force { SERIAL.force_unlock() } else { SERIAL.lock() };

        write(&mut *serial, args, newline);

        if no_cons {
            return;
        }

        let mut cons_cell = if force { CONSOLE.force_unlock() } else { CONSOLE.lock() };

        if let Some(console) = cons_cell.get_mut() {
            write(console, args, newline);
        }
    });
}

fn write(output: &mut impl Write, args: &fmt::Arguments, newline: bool) {
    if newline {
        writeln!(output, "{}", args).unwrap();
    } else {
        write!(output, "{}", args).unwrap();
    }
}

#[macro_export]
macro_rules! trace {
    () => {
        $crate::println!("{}:{}", module_leaf!(), line!())
    };
    ($e:expr) => {
        $crate::println!("{}: {}", module_leaf!(), &$e)
    };
    ($e:ident) => {
        $crate::println!("{}: {} = {:#?}", module_leaf!(), stringify!($e), &$e)
    };
    ($($args:tt)*) => {
        $crate::println!("{}: {}", module_leaf!(), &format_args!($($args)*))
    }
}

macro_rules! module_leaf {
    () => {
        module_path!().split("::").last().unwrap()
    };
}
