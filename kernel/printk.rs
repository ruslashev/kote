// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        use $crate::serial::SERIAL_LOCK;
        use $crate::console::CONSOLE;

        let mut serial = SERIAL_LOCK.guard();
        let mut cons_cell = CONSOLE.guard();
        let console = cons_cell.get_mut().unwrap();

        writeln!(&mut serial, "{}", format_args!($($arg)*)).unwrap();
        writeln!(console, "{}", format_args!($($arg)*)).unwrap();
    });
}

// Copied from std
#[macro_export]
macro_rules! dbg {
    () => {
        $crate::println!("[{}:{}]", file!(), line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
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
