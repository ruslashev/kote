// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        use $crate::serial::SERIAL_LOCK;
        use $crate::console::CONSOLE;

        // 1. Lock after constructing parameters to avoid a deadlock on interrupt,
        // 2. Match is used here because we can't create a `let args = format_args!($($arg)*)`
        //    binding, see https://stackoverflow.com/a/48732525/1063961
        match format_args!($($arg)*) {
            args => {
                let mut serial = SERIAL_LOCK.guard();
                let mut cons_cell = CONSOLE.guard();
                let console = cons_cell.get_mut().unwrap();

                writeln!(&mut serial, "{}", &args).unwrap();
                writeln!(console, "{}", args).unwrap();
            }
        }
    });
}

// Copied from std
#[macro_export]
macro_rules! dbg {
    () => {
        $crate::println!("[{}:{}]", file!(), line!())
    };
    ($val:expr $(,)?) => {
        // Same as in println!() above, use of `match` here is intentional because it affects the
        // lifetimes of temporaries.
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
