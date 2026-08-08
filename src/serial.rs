use uart_16550::backend;

lazy_static::lazy_static! {
    pub static ref SERIAL1: spin::Mutex<uart_16550::Uart16550Tty<backend::PioBackend>> = spin::Mutex::new(unsafe {
        uart_16550::Uart16550Tty::new_port(0x3F8, uart_16550::Config::default())
            .expect("failed to init uart")
    });
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("failed to print to serial1")
    });
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
