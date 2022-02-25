// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Usage of `match expr { x => { … } }` in these macros is intentional because it affects the
// lifetimes of temporaries. For example, we can't create a `let args = format_args!($($arg)*)`
// binding. Its usage here is similar to an absent `let x = expr in { … }` construct.
// See https://stackoverflow.com/a/48732525/1063961

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        use $crate::serial::SERIAL_LOCK;
        use $crate::console::CONSOLE;
        use $crate::arch::interrupts;

        interrupts::disable();

        match format_args!($($arg)*) {
            args => {
                let mut serial = SERIAL_LOCK.guard();
                let mut cons_cell = CONSOLE.guard();
                let console = cons_cell.get_mut().unwrap();

                writeln!(&mut serial, "{}", &args).unwrap();
                writeln!(console, "{}", args).unwrap();
            }
        }

        interrupts::enable();
    });
}

#[macro_export]
macro_rules! println_force {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        use $crate::serial::SERIAL_LOCK;
        use $crate::console::CONSOLE;
        use $crate::arch::interrupts;

        interrupts::disable();

        match format_args!($($arg)*) {
            args => {
                let mut serial = SERIAL_LOCK.force_unlock();
                let mut cons_cell = CONSOLE.force_unlock();
                let console = cons_cell.get_mut().unwrap();

                writeln!(&mut serial, "{}", &args).unwrap();
                writeln!(console, "{}", args).unwrap();
            }
        }

        interrupts::enable();
    });
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
