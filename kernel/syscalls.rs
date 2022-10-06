// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::fmt;

use crate::sched;

#[repr(C, packed)]
pub struct SyscallArgs {
    number: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
}

impl fmt::Display for SyscallArgs {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let num = self.number;
        let arg1 = self.arg1;
        let arg2 = self.arg2;
        let arg3 = self.arg3;
        let arg4 = self.arg4;

        write!(f, "num={} args: {:x} {:x} {:x} {:x}", num, arg1, arg2, arg3, arg4)
    }
}

#[no_mangle]
pub extern "C" fn syscall_dispatch(args: &SyscallArgs) {
    trace!("{}", args);

    sched::next();
}
