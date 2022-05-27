// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::lazy::OnceCell;

use crate::bootloader::BootloaderInfo;
use crate::panic::panic_no_graphics;
use crate::spinlock::SpinlockMutex;

pub static CONSOLE: SpinlockMutex<OnceCell<Console>> = SpinlockMutex::new(OnceCell::new());

const FONT: &[u8] = include_bytes!("../Lat7-Fixed14.psf");
const FB_LUT: [[u32; 8]; 2usize.pow(8)] = compute_fb_lut();
const COLOR_BG: u32 = 0x000000;
const COLOR_FG: u32 = 0xe5e5e5;

#[derive(Debug)]
pub struct Console {
    fb: Framebuffer,
    font: Font,
    cursor_x: u32,
    cursor_y: u32,
    width: u32,
    height: u32,
}

impl Console {
    fn new(fb_addr: usize, info: &BootloaderInfo) -> Self {
        let font = Font::from_bytes(FONT);
        let width = info.framebuffer.width / font.width;
        let height = info.framebuffer.height / font.height as u32;

        Console {
            fb: Framebuffer::new(fb_addr, info),
            font,
            cursor_x: 0,
            cursor_y: 0,
            width,
            height,
        }
    }

    fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.write_byte(b);
        }
    }

    fn write_byte(&mut self, b: u8) {
        if b == b'\n' {
            self.newline();
            return;
        }

        if self.cursor_x == self.width - 1 {
            self.newline();
        }

        let x = self.cursor_x * self.font.width;
        let y = self.cursor_y * self.font.height as u32;

        self.fb.draw_char(b, &self.font, x, y);

        self.cursor_x += 1;
    }

    fn newline(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;

        if self.cursor_y >= self.height {
            self.cursor_y -= 1;
            self.shift_up();
        }
    }

    fn shift_up(&mut self) {
        let bpp = self.fb.bytes_per_pixel as usize;
        let row = bpp * (self.width * self.font.width) as usize;
        let src = (self.fb.addr + row * self.font.height) as *const u8;
        let dst = self.fb.addr as *mut u8;
        let cnt = row * (self.fb.height as usize - self.font.height);

        unsafe {
            core::ptr::copy(src, dst, cnt);
        }
    }
}

impl core::fmt::Write for Console {
    // NOTE: Same as `Serial`, by itself makes no exclusivity guarantees
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_str(s);

        Ok(())
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
    fn new(fb_addr: usize, info: &BootloaderInfo) -> Self {
        let fb = &info.framebuffer;

        Framebuffer {
            addr: fb_addr,
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
            panic_no_graphics("Font magic mismatch");
        }

        Font {
            width: 8,
            height: bytes[3] as usize,
            glyphs: &bytes[4..],
        }
    }
}

pub fn init(fb_addr: usize, info: &BootloaderInfo) {
    let cell = CONSOLE.guard();
    cell.set(Console::new(fb_addr, info)).unwrap();
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
