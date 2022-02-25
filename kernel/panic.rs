// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::fmt::Write;
use core::panic::PanicInfo;

use crate::serial::SERIAL_LOCK;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    print_backtrace!();

    loop {}
}

/// Panicking at the earliest boot stage when there's no serial and graphics. Don't really know
/// what to do here, since we can't communicate the error.
pub fn panic_no_serial(_message: &str) {
    // Don't abort here and continue, pretending serial is not broken
}

/// Panic at early boot stage when there's serial but no graphics. Print to serial.
pub fn panic_no_graphics(message: &str) {
    let mut serial = SERIAL_LOCK.guard();
    writeln!(&mut serial, "{}", message).unwrap();
}
