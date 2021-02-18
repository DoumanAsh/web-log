# web-log

![Rust](https://github.com/DoumanAsh/uuid/workflows/Rust/badge.svg?branch=master)
[![Crates.io](https://img.shields.io/crates/v/web-log.svg)](https://crates.io/crates/web-log)
[![Documentation](https://docs.rs/web-log/badge.svg)](https://docs.rs/crate/web-log/)

Minimal wrapper over browser console to provide printing facilities

## Features:

- `std` - Enables `std::io::Write` implementation.

## Usage

```rust,no_run
use web_log::{ConsoleType, Console};

use core::fmt::Write;

let mut writer = Console::new(ConsoleType::Info);
let _ = write!(writer, "Hellow World!");
drop(writer); //or writer.flush();

web_log::println!("Hello via macro!");
web_log::eprintln!("Error via macro!");
```

