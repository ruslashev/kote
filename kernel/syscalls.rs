// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::convert::Infallible;
use core::ops::{ControlFlow, FromResidual, Try};
use core::{fmt, slice, str};

use crate::arch::RegisterFrame;
use crate::mm::types::{Address, RootPageDirOps, VirtAddr};
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

impl From<RegisterFrame> for SyscallArgs {
    fn from(regs: RegisterFrame) -> Self {
        Self {
            number: regs.rax,
            arg1: regs.rdi,
            arg2: regs.rsi,
            arg3: regs.rdx,
            arg4: regs.r10,
        }
    }
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

#[must_use]
enum NumericResult<T> {
    Ok(T),
    Err(u64),
}

trait ConvertToNumericResult<T> {
    fn convert_err(self, err: u64) -> NumericResult<T>;
}

impl<T, E> ConvertToNumericResult<T> for Result<T, E> {
    fn convert_err(self, err: u64) -> NumericResult<T> {
        match self {
            Ok(t) => NumericResult::Ok(t),
            Err(_) => NumericResult::Err(err),
        }
    }
}

impl<T> Try for NumericResult<T> {
    type Output = T;
    type Residual = NumericResult<Infallible>;

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        NumericResult::Ok(output)
    }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            NumericResult::Ok(t) => ControlFlow::Continue(t),
            NumericResult::Err(num) => ControlFlow::Break(NumericResult::Err(num)),
        }
    }
}

impl<T> FromResidual for NumericResult<T> {
    #[inline]
    fn from_residual(residual: NumericResult<Infallible>) -> Self {
        match residual {
            // This match arm shouldn't be needed, but rustc complains without it
            NumericResult::Ok(_) => unreachable!(),
            NumericResult::Err(num) => NumericResult::Err(num),
        }
    }
}

impl FromResidual<NumericResult<Infallible>> for u64 {
    #[inline]
    fn from_residual(residual: NumericResult<Infallible>) -> Self {
        match residual {
            NumericResult::Ok(_) => unreachable!(),
            NumericResult::Err(num) => num,
        }
    }
}

#[no_mangle]
pub extern "C" fn syscall_dispatch(regs: &RegisterFrame) -> u64 {
    let args = SyscallArgs::from(*regs);

    sched::current().registers = *regs;

    trace!("{}", args);

    match args.number {
        SYSC_YIELD => sched::next(),
        SYSC_WRITE => write(&args),
        _ => {
            trace!("invalid syscall number");
            SYSR_ERR_BAD_ARGS
        }
    }
}

fn write(args: &SyscallArgs) -> u64 {
    let mut root_dir = sched::current().root_dir;
    let addr = args.arg1;
    let size = args.arg2 as usize;
    let from = VirtAddr::from_u64(addr);

    if !root_dir.is_region_user_accessible(from, from + size) {
        return SYSR_ERR_NO_PERMISSIONS;
    }

    let ptr = addr as *const u8;

    unsafe {
        let slice = slice::from_raw_parts(ptr, size);
        let string = str::from_utf8(slice).convert_err(SYSR_ERR_BAD_ARGS)?;

        print!("{}", string);
    }

    SYSR_OK
}
