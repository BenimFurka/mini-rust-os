use crate::vga_buffer::Color;
use crate::print;
use crate::commands;

pub fn handle_command(input: &[u8]) {
    let input_str = match core::str::from_utf8(input) {
        Ok(s) => s.trim(),
        Err(_) => {
            print!(("\nInvalid UTF-8 input"), fg: Color::Red);
            return;
        }
    };

    if input_str.is_empty() {
        return;
    }

    let mut parts = input_str.split_whitespace();
    let command = match parts.next() {
        Some(cmd) => cmd,
        None => return,
    };
    
    let mut args = [""; 16];
    let mut arg_count = 0;
    for (i, arg) in parts.enumerate().take(15) {
        args[i] = arg;
        arg_count = i + 1;
    }

    commands::handle_command(command, &args[..arg_count]);
}
