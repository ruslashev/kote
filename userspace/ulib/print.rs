// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::fmt::{self, Write};

struct SyscallWriter;

impl fmt::Write for SyscallWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        super::write(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        $crate::print::do_print(&format_args_nl!($($arg)*))
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::print::do_print(&format_args!($($arg)*))
    }
}

pub fn do_print(args: &fmt::Arguments) {
    write!(SyscallWriter, "{}", args).unwrap();
}
