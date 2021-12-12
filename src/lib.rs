//! Minimal wrapper over browser console to provide printing facilities
//!
//! ## Features:
//!
//! - `std` - Enables `std::io::Write` implementation.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use web_log::{ConsoleType, Console};
//!
//! use core::fmt::Write;
//!
//! let mut writer = Console::new(ConsoleType::Info);
//! let _ = write!(writer, "Hellow World!");
//! drop(writer); //or writer.flush();
//!
//! web_log::println!("Hello via macro!");
//! web_log::eprintln!("Error via macro!");
//! ```

#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(test))]
use wasm_bindgen::prelude::wasm_bindgen;

use core::{cmp, ptr, mem, fmt};

#[cfg(not(test))]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn warn(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn info(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn debug(s: &str);
}

#[cfg(test)]
fn error(_: &str) {
}

#[cfg(test)]
fn warn(_: &str) {
}

#[cfg(test)]
fn info(_: &str) {
}

#[cfg(test)]
fn debug(_: &str) {
}

const BUFFER_CAPACITY: usize = 4096;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
///Specifies method of writing into console.
pub enum ConsoleType {
    ///Uses `console.error`
    Error,
    ///Uses `console.warn`
    Warn,
    ///Uses `console.info`
    Info,
    ///Uses `console.debug`
    Debug,
}

///Wrapper over browser's console
///
///On `Drop` performs `flush` or requires manual `flush` for written to be printed in the console.
///Buffer capacity is 4096 bytes.
///In case of overflow it dumps existing data to the console and overwrites with rest of it.
pub struct Console {
    typ: ConsoleType,
    buffer: mem::MaybeUninit<[u8; BUFFER_CAPACITY]>,
    len: usize,
}

impl Console {
    ///Creates new instance
    pub const fn new(typ: ConsoleType) -> Self {
        Self {
            typ,
            buffer: mem::MaybeUninit::uninit(),
            len: 0,
        }
    }

    #[inline(always)]
    ///Returns content of written buffer.
    pub fn buffer(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self.buffer.as_ptr() as *const u8, self.len)
        }
    }

    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buffer.as_mut_ptr() as _
    }

    #[inline(always)]
    ///Flushes internal buffer, if any data is available.
    ///
    ///Namely it dumps stored data in buffer via Console.
    ///And resets buffered length to 0.
    pub fn flush(&mut self) {
        if self.len > 0 {
            self.inner_flush();
        }
    }

    fn inner_flush(&mut self) {
        let text = unsafe {
            core::str::from_utf8_unchecked(self.buffer())
        };
        match self.typ {
            ConsoleType::Error => error(text),
            ConsoleType::Warn => warn(text),
            ConsoleType::Info => info(text),
            ConsoleType::Debug => debug(text),
        }

        self.len = 0;
    }

    #[inline]
    fn copy_data<'a>(&mut self, text: &'a [u8]) -> &'a [u8] {
        let mut write_len = cmp::min(BUFFER_CAPACITY.saturating_sub(self.len), text.len());

        #[inline(always)]
        fn is_char_boundary(text: &[u8], idx: usize) -> bool {
            if idx == 0 {
                return true;
            }

            match text.get(idx) {
                None => idx == text.len(),
                Some(&byte) => (byte as i8) >= -0x40
            }
        }

        #[inline(never)]
        #[cold]
        fn shift_by_char_boundary(text: &[u8], mut size: usize) -> usize {
            while !is_char_boundary(text, size) {
                size -= 1;
            }
            size
        }

        if !is_char_boundary(text, write_len) {
            //0 is always char boundary so 0 - 1 is impossible
            write_len = shift_by_char_boundary(text, write_len - 1);
        }

        unsafe {
            ptr::copy_nonoverlapping(text.as_ptr(), self.as_mut_ptr().add(self.len), write_len);
        }
        self.len += write_len;
        &text[write_len..]
    }

    ///Writes supplied text to the buffer.
    ///
    ///On buffer overflow, data is logged via `Console`
    ///and buffer is filled with the rest of `data`
    pub fn write_data(&mut self, mut data: &[u8]) {
        loop {
            data = self.copy_data(data);

            if data.is_empty() {
                break;
            } else {
                self.flush();
            }
        }
    }
}

impl fmt::Write for Console {
    #[inline]
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.write_data(text.as_bytes());

        Ok(())
    }
}

#[cfg(feature = "std")]
impl std::io::Write for Console {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_data(buf);
        Ok(buf.len())
    }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> {
        self.flush();
        Ok(())
    }
}

impl Drop for Console {
    #[inline]
    fn drop(&mut self) {
        self.flush();
    }
}

#[macro_export]
///`println` alternative to write message with INFO priority.
macro_rules! println {
    () => {{
        $crate::println!(" ");
    }};
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut writer = $crate::Console::new($crate::ConsoleType::Info);
        let _ = write!(writer, $($arg)*);
        drop(writer);
    }}
}

#[macro_export]
///`eprintln` alternative to write message with ERROR priority.
macro_rules! eprintln {
    () => {{
        $crate::println!(" ");
    }};
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut writer = $crate::Console::new($crate::ConsoleType::Error);
        let _ = write!(writer, $($arg)*);
        drop(writer);
    }}
}

#[cfg(test)]
mod tests {
    use super::{Console, ConsoleType};
    const DATA: &str = "1234567891";

    #[test]
    fn should_normal_write() {
        let mut writer = Console::new(ConsoleType::Warn);

        assert_eq!(writer.typ, ConsoleType::Warn);

        let data = DATA.as_bytes();

        writer.write_data(data);
        assert_eq!(writer.len, data.len());
        assert_eq!(writer.buffer(), data);

        writer.write_data(b" ");
        writer.write_data(data);
        let expected = format!("{} {}", DATA, DATA);
        assert_eq!(writer.len, expected.len());
        assert_eq!(writer.buffer(), expected.as_bytes());
    }

    #[test]
    fn should_handle_write_overflow() {
        let mut writer = Console::new(ConsoleType::Warn);
        let data = DATA.as_bytes();

        //BUFFER_CAPACITY / DATA.len() = 148.xxx
        for idx in 1..=409 {
            writer.write_data(data);
            assert_eq!(writer.len, data.len() * idx);
        }

        writer.write_data(data);
        assert_eq!(writer.len, 4);
        writer.flush();
        assert_eq!(writer.len, 0);
    }

    #[test]
    fn should_handle_write_overflow_outside_of_char_boundary() {
        let mut writer = Console::new(ConsoleType::Warn);
        let data = DATA.as_bytes();

        for idx in 1..=409 {
            writer.write_data(data);
            assert_eq!(writer.len, data.len() * idx);
        }

        writer.write_data(b"1234");
        assert_eq!(4094, writer.len);
        let unicode = "ロリ";
        writer.write_data(unicode.as_bytes());
        assert_eq!(writer.len, unicode.len());
        assert_eq!(writer.buffer(), unicode.as_bytes());
    }
}
