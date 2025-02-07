use crate::vga_buffer::Color;

macro_rules! print {
    (($($arg:tt)*)) => ({
        $crate::vga_buffer::print(format_args!($($arg)*), None, None);
    });
    (($($arg:tt)*), fg: $fg:expr) => ({
        $crate::vga_buffer::print(format_args!($($arg)*), Some($fg), None);
    });
    (($($arg:tt)*), bg: $bg:expr) => ({
        $crate::vga_buffer::print(format_args!($($arg)*), None, Some($bg));
    });
    (($($arg:tt)*), fg: $fg:expr, bg: $bg:expr) => ({
        $crate::vga_buffer::print(format_args!($($arg)*), Some($fg), Some($bg));
    });
}


pub fn handle_name(args: &[u8]) {
    ///let args_str = core::str::from_utf8(args).unwrap_or("").trim();
    //unsafe { crate::USER_NAME = args_str; }
    print!(("\nAnd what?"), fg: Color::Black, bg: Color::White);
    print!((""), bg: Color::Black);
}