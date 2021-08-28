use crate::serial;
use crate::spinlock::Spinlock;

pub static PRINT_LOCK: Spinlock = Spinlock::new();

pub struct Serial<'a>(&'a Spinlock);

impl Serial<'_>
{
	pub fn get() -> Self
	{
		Serial { 0: &PRINT_LOCK }
	}
}

impl core::fmt::Write for Serial<'_>
{
	fn write_str(&mut self, s: &str) -> core::fmt::Result
	{
		self.0.lock();

		for b in s.bytes() {
			serial::write_byte(b);
		}

		self.0.unlock();

		Ok(())
	}
}

#[macro_export]
macro_rules! printk {
	($($arg:tt)*) => {
		use core::fmt::Write;

		$crate::printk::Serial::get().write_fmt(format_args!($($arg)*)).unwrap();
	};
}
