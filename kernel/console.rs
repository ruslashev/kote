// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::lazy::OnceCell;
use crate::multiboot::BootloaderInfo;
use crate::spinlock::SpinlockMutex;

static CONSOLE: SpinlockMutex<OnceCell<Console>> = SpinlockMutex::new(OnceCell::new());

#[derive(Debug)]
struct Console {
    addr: usize,
    width: u32,
    height: u32,
    pitch: u32,
    bytes_per_pixel: u8,
}

impl Console {
    fn from_info(info: &BootloaderInfo) -> Self {
        let fb = &info.framebuffer;

        Console {
            addr: usize::try_from(fb.addr).unwrap(),
            width: fb.width,
            height: fb.height,
            pitch: fb.pitch,
            bytes_per_pixel: fb.bpp / 8,
        }
    }

    fn putpixel(&self, x: u32, y: u32, color: u32) {
        let pos = y * self.pitch + x * self.bytes_per_pixel as u32;
        let ptr = (self.addr + pos as usize) as *mut u32;

        unsafe {
            ptr.write(color);
        }
    }
}

pub fn init(info: &BootloaderInfo) {
    let cell = CONSOLE.guard();

    cell.set(Console::from_info(info)).unwrap();

    let cons = cell.get().unwrap();

    let mut sx;
    let sz = 50;

    for y in 0..sz {
        sx = sz * 0;
        for x in 0..sz {
            cons.putpixel(sx + x, y, u32::max_value());
        }

        sx = sz * 1;
        for x in 0..sz {
            cons.putpixel(sx + x, y, 0xff0000);
        }

        sx = sz * 2;
        for x in 0..sz {
            cons.putpixel(sx + x, y, 0x00ff00);
        }

        sx = sz * 3;
        for x in 0..sz {
            cons.putpixel(sx + x, y, 0x0000ff);
        }
    }
}
