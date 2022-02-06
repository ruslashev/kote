// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::multiboot::BootloaderInfo;

struct TextBuffer {
    addr: usize,
    width: u32,
    height: u32,
    pitch: u32,
    bytes_per_pixel: u8,
}

impl TextBuffer {
    const fn uninit() -> Self {
        TextBuffer {
            addr: 0,
            width: 0,
            height: 0,
            pitch: 0,
            bytes_per_pixel: 0,
        }
    }

    fn from_info(info: &BootloaderInfo) -> Self {
        let fb = &info.framebuffer;

        TextBuffer {
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

static mut BUFFER: TextBuffer = TextBuffer::uninit();

pub fn init(info: &BootloaderInfo) {
    unsafe {
        BUFFER = TextBuffer::from_info(info);

        let mut sx;
        let sz = 50;

        for y in 0..sz {
            sx = sz * 0;
            for x in 0..sz {
                BUFFER.putpixel(sx + x, y, u32::max_value());
            }

            sx = sz * 1;
            for x in 0..sz {
                BUFFER.putpixel(sx + x, y, 0xff0000);
            }

            sx = sz * 2;
            for x in 0..sz {
                BUFFER.putpixel(sx + x, y, 0x00ff00);
            }

            sx = sz * 3;
            for x in 0..sz {
                BUFFER.putpixel(sx + x, y, 0x0000ff);
            }
        }
    }
}
