use crate::arch::io::{inb, outb};
use crate::panic::panic_early;

const COM1_PORT: u16 = 0x3f8;

const COM_THR: u16 = 0; // Out: Transmitter Holding Register (when DLAB = 0)
const COM_RBR: u16 = 0; // In:  Receiver buffer              (when DLAB = 0)
const COM_IER: u16 = 1; // I/O: Interrupt Enable Register    (when DLAB = 0)
const COM_DLL: u16 = 0; // I/O: Divisor Latch Low            (when DLAB = 1)
const COM_DLM: u16 = 1; // I/O: Divisor Latch High           (when DLAB = 1)
const COM_FCR: u16 = 2; // Out: FIFO Control Register
const COM_LCR: u16 = 3; // I/O: Line Control Register
const COM_MCR: u16 = 4; // I/O: Modem Control Register
const COM_LSR: u16 = 5; // In:  Line Status Register

const COM_LCR_DLAB_BIT: u8 = 0x80; // Divisor latch access bit
const COM_IER_RDI_BIT: u8 = 0x1; // Enable receiver data interrupt

const COM_LSR_DATA: u8 = 0x01; // Data ready
const COM_LSR_THRE: u8 = 0x20; // Transmitter holding register empty

pub fn init() {
    // Turn off the FIFO
    outb(COM1_PORT + COM_FCR, 0);

    // Disable interrupts
    outb(COM1_PORT + COM_IER, 0);

    // Enable DLAB
    outb(COM1_PORT + COM_LCR, COM_LCR_DLAB_BIT);

    // Set speed to 38400 baud (115200 / 38400 = 3)
    outb(COM1_PORT + COM_DLL, 3);
    outb(COM1_PORT + COM_DLM, 0);

    // 8 data bits, 1 stop bit, no parity, disable DLAB
    outb(COM1_PORT + COM_LCR, 0b00000011);

    // FIFO: enable, clear, 14-byte size
    outb(COM1_PORT + COM_FCR, 0b11000111);

    // Test: enable loopback mode
    outb(COM1_PORT + COM_MCR, 0b00011110);

    // Send a byte
    outb(COM1_PORT + COM_THR, 0x80);

    if inb(COM1_PORT + COM_RBR) != 0x80 {
        panic_early("Failed to init serial: didn't return the same byte as sent");
    }

    // Disable loopback, enable aux bits 1, 2
    outb(COM1_PORT + COM_MCR, 0b00001111);

    if inb(COM1_PORT + COM_LSR) == 0xff {
        panic_early("Failed to init serial: LSR returned 0xFF");
    }

    // Enable receiver interrupts
    // outb(COM1_PORT + COM_IER, COM_IER_RDI);
}

pub fn read_byte() -> u8 {
    while inb(COM1_PORT + COM_LSR) & COM_LSR_DATA == 0 {}

    return inb(COM1_PORT + COM_RBR);
}

pub fn write_byte(byte: u8) {
    while inb(COM1_PORT + COM_LSR) & COM_LSR_THRE == 0 {}

    outb(COM1_PORT + COM_THR, byte);
}
