// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::lazy::OnceCell;

use crate::multiboot::BootloaderInfo;
use crate::panic::panic_early;
use crate::spinlock::SpinlockMutex;

static CONSOLE: SpinlockMutex<OnceCell<Console>> = SpinlockMutex::new(OnceCell::new());

const FONT: &[u8] = include_bytes!("../Lat7-Fixed14.psf");
const FB_LUT: [[u32; 8]; 2usize.pow(8)] = compute_fb_lut();
const COLOR_BG: u32 = 0x000000;
const COLOR_FG: u32 = 0xe5e5e5;

#[derive(Debug)]
struct Console {
    fb: Framebuffer,
    font: Font,
    cursor_x: u32,
    cursor_y: u32,
}

impl Console {
    fn from_info(info: &BootloaderInfo) -> Self {
        Console {
            fb: Framebuffer::from_info(info),
            font: Font::from_bytes(FONT),
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    fn write_byte(&mut self, b: u8) {
        let x = self.cursor_x * self.font.width;
        let y = self.cursor_y * self.font.height as u32;

        self.fb.draw_char(b, &self.font, x, y);

        self.cursor_x += 1;
    }

    fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.write_byte(b);
        }
    }
}

#[derive(Debug)]
struct Framebuffer {
    addr: usize,
    width: u32,
    height: u32,
    pitch: u32,
    bytes_per_pixel: u8,
}

impl Framebuffer {
    fn from_info(info: &BootloaderInfo) -> Self {
        let fb = &info.framebuffer;

        Framebuffer {
            addr: usize::try_from(fb.addr).unwrap(),
            width: fb.width,
            height: fb.height,
            pitch: fb.pitch,
            bytes_per_pixel: fb.bpp / 8,
        }
    }

    fn draw_pixel(&self, x: u32, y: u32, color: u32) {
        let pos = y * self.pitch + x * self.bytes_per_pixel as u32;
        let ptr = (self.addr + pos as usize) as *mut u32;

        unsafe {
            ptr.write(color);
        }
    }

    fn draw_char(&self, b: u8, font: &Font, x: u32, y: u32) {
        let offset = font.height * b as usize;
        let glyph = &font.glyphs[offset..offset + font.height];
        let mut dy = y;

        for byte in glyph {
            let pos = dy * self.pitch + x * self.bytes_per_pixel as u32;
            let ptr = (self.addr + pos as usize) as *mut [u32; 8];
            let row = &FB_LUT[*byte as usize];

            unsafe {
                ptr.write(*row);
            }

            dy += 1;
        }
    }
}

#[derive(Debug)]
struct Font {
    width: u32,
    height: usize,
    glyphs: &'static [u8],
}

impl Font {
    fn from_bytes(bytes: &'static [u8]) -> Self {
        if bytes[0..=1] != [0x36, 0x04] {
            panic_early("Font magic mismatch");
        }

        Font {
            width: 8,
            height: bytes[3] as usize,
            glyphs: &bytes[4..],
        }
    }
}

pub fn init(info: &BootloaderInfo) {
    let mut cell = CONSOLE.guard();

    cell.set(Console::from_info(info)).unwrap();

    let cons = cell.get_mut().unwrap();

    cons.write_str("Hello, world!");
}

const fn compute_fb_lut() -> [[u32; 8]; 2usize.pow(8)] {
    let mut lut = [[0; 8]; 2usize.pow(8)];
    let num_combinations = 2usize.pow(8);
    let mut byte = 0;

    while byte < num_combinations {
        let mut mask_idx = 0;

        while mask_idx < 8 {
            let mask = 1 << mask_idx;
            let bit = byte & mask;
            let set = bit != 0;

            lut[byte][7 - mask_idx] = if set { COLOR_FG } else { COLOR_BG };

            mask_idx += 1;
        }

        byte += 1;
    }

    lut
}
