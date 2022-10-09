// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::{fmt, slice, str};

use crate::mm::types::{RootPageDirOps, VirtAddr, Address};
use crate::sched;

const SYSC_YIELD: u64 = 0;
const SYSC_WRITE: u64 = 1;

const SYSR_OK: u64 = 0;
const SYSR_ERR_NO_PERMISSIONS: u64 = 1;
const SYSR_ERR_BAD_ARGS: u64 = 2;

#[repr(C, packed)]
pub struct SyscallArgs {
    number: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
}

impl fmt::Display for SyscallArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let num = self.number;
        let arg1 = self.arg1;
        let arg2 = self.arg2;
        let arg3 = self.arg3;
        let arg4 = self.arg4;

        write!(f, "num={} args: {:x} {:x} {:x} {:x}", num, arg1, arg2, arg3, arg4)
    }
}

#[no_mangle]
pub extern "C" fn syscall_dispatch(args: &SyscallArgs) -> u64 {
    trace!("{}", args);

    match args.number {
        SYSC_YIELD => sched::next(),
        SYSC_WRITE => write(args),
        _ => {
            trace!("invalid syscall number");
            SYSR_ERR_BAD_ARGS
        }
    }
}

fn write(args: &SyscallArgs) -> u64 {
    let mut root_dir = sched::current().unwrap().root_dir;
    let addr = args.arg1;
    let size = args.arg2 as usize;
    let from = VirtAddr::from_u64(addr);

    if !root_dir.is_region_user_accessible(from, from + size) {
        return SYSR_ERR_NO_PERMISSIONS;
    }

    let ptr = addr as *const u8;

    unsafe {
        let slice = slice::from_raw_parts(ptr, size);
        let string = match str::from_utf8(slice) {
            Ok(string) => string,
            _ => return SYSR_ERR_BAD_ARGS,
        };

        println!("{}", string);
    }

    SYSR_OK
}
