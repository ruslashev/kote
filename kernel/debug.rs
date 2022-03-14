// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#[macro_export]
macro_rules! print_backtrace {
    () => {
        use $crate::arch::backtrace::Backtrace;

        println!("Backtrace:");

        for (i, addr) in Backtrace::from_here().enumerate() {
            println!("{:>2}) {:#x}", i + 1, addr);
        }
    };
}
