use crate::{commands::{calc::handle_calc, echo::handle_echo, help::handle_help, name::handle_name}, vga_buffer::{self, Color}};
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

pub fn handle_command(input: &[u8]) {
    let mut name_end = 0;
    while name_end < input.len() && input[name_end] != b' ' {
        name_end += 1;
    }

    let name = &input[0..name_end];
    let args = if name_end < input.len() && input[name_end] == b' ' {
        &input[name_end + 1..]
    } else {
        &[]
    };

    match core::str::from_utf8(name).unwrap_or("") {
        "calc" => handle_calc(args),
        "help" => handle_help(args),
        "echo" => handle_echo(args),
        "name" => handle_name(args),
        "clear" => vga_buffer::clear_screen(),
        _ => print!(("\nUnknown command: {}", core::str::from_utf8(name).unwrap_or("")), fg: Color::Yellow),
    }
}
