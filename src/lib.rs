#![no_std]
#![no_main]

use handle_command::handle_command;
use vga_buffer::{Color, WRITER};

mod handle_command;
pub mod commands;
mod vga_buffer;
mod spin;

extern crate rlibc;
extern crate volatile;

extern "C" {
    fn read_keyboard_input() -> u8;
}
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


const INPUT_BUFFER_SIZE: usize = 128;
pub static mut USER_NAME: &str = "User";
static mut INPUT_BUFFER: [u8; INPUT_BUFFER_SIZE] = [0; INPUT_BUFFER_SIZE];
static mut INPUT_BUFFER_INDEX: usize = 0;

const SCANCODE_TABLE: [char; 128] = [
    '?', '?', '1', '2', '3', '4', '5', '6', 
    '7', '8', '9', '0', '-', '=', '?', '?', 
    'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 
    'o', 'p', '[', ']', '?', '?', 'a', 's', 
    'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', 
    '\'', '`', '?', '\\', 'z', 'x', 'c', 'v', 
    'b', 'n', 'm', ',', '.', '/', '?', '?', 
    '?', ' ', '?', '?', '?', '?', '?', '?', 
    '?', '?', '?', '?', '?', '?', '?', '?', 
    '?', '?', '?', '?', '?', '?', '?', '?',
    '?', '?', '?', '?', '?', '?', '?', '?', 
    '?', '?', '?', '?', '?', '?', '?', '?', 
    '?', '?', '?', '?', '?', '?', '?', '?', 
    '?', '?', '?', '?', '?', '?', '?', '?', 
    '?', '?', '?', '?', '?', '?', '?', '?', 
    '?', '?', '?', '?', '?', '?', '?', '?',
];

const SCANCODE_TABLE_SHIFT: [char; 128] = [
    '?', '?', '!', '@', '#', '$', '%', '^', 
    '&', '*', '(', ')', '_', '+', '?', '?', 
    'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 
    'O', 'P', '{', '}', '?', '?', 'A', 'S', 
    'D', 'F', 'G', 'H', 'J', 'K', 'L', ':',
    '"', '~', '?', '|', 'Z', 'X', 'C', 'V', 
    'B', 'N', 'M', '<', '>', '?', '?', '?',
    '?', ' ', '?', '?', '?', '?', '?', '?', 
    '?', '?', '?', '?', '?', '?', '?', '?',
    '?', '?', '?', '?', '?', '?', '?', '?', 
    '?', '?', '?', '?', '?', '?', '?', '?', 
    '?', '?', '?', '?', '?', '?', '?', '?',
    '?', '?', '?', '?', '?', '?', '?', '?',
    '?', '?', '?', '?', '?', '?', '?', '?', 
    '?', '?', '?', '?', '?', '?', '?', '?', 
    '?', '?', '?', '?', '?', '?', '?', '?', 
];
static mut SHIFT_PRESSED: bool = false;
static mut CAPS_LOCK_PRESSED: bool = false;

pub fn handle_input(scancode: u8) {
    unsafe {
        match scancode {
            0x3A => {
                CAPS_LOCK_PRESSED = !CAPS_LOCK_PRESSED;
            }
            0x2A | 0x36 => {
                SHIFT_PRESSED = true;
            }
            0xAA | 0xB6 => {
                SHIFT_PRESSED = false;
            }
            0x0E => {
                if INPUT_BUFFER_INDEX > 0 {
                    INPUT_BUFFER_INDEX -= 1;
                    WRITER.delete_char();
                }
            }
            0x1C => {
                if INPUT_BUFFER_INDEX > 0 {
                    let input_str = core::str::from_utf8(&INPUT_BUFFER[0..INPUT_BUFFER_INDEX])
                        .unwrap_or("")
                        .trim();

                    handle_command(input_str.as_bytes());

                    INPUT_BUFFER_INDEX = 0;
                }

                print!(("\n{} >> ", USER_NAME), fg: Color::LightGreen); 
            }
            _ => {
                if scancode < 128 {
                    let character = if SHIFT_PRESSED {
                        SCANCODE_TABLE_SHIFT[scancode as usize]
                    } else {
                        SCANCODE_TABLE[scancode as usize]
                    };

                    if character.is_ascii_alphabetic() {
                        if CAPS_LOCK_PRESSED {
                            INPUT_BUFFER[INPUT_BUFFER_INDEX] = character.to_ascii_uppercase() as u8;
                        } else {
                            INPUT_BUFFER[INPUT_BUFFER_INDEX] = character as u8;
                        }
                    } else {
                        INPUT_BUFFER[INPUT_BUFFER_INDEX] = character as u8;
                    }

                    if character != '?' && INPUT_BUFFER_INDEX < INPUT_BUFFER_SIZE {
                        INPUT_BUFFER_INDEX += 1;
                        print!((
                            "{}",
                            if CAPS_LOCK_PRESSED {
                                character.to_ascii_uppercase()
                            } else {
                                character
                            }),
                            fg: Color::LightGray
                        );
                    }

                    if scancode == 0x2A || scancode == 0x36 {
                        return; 
                    }

                    SHIFT_PRESSED = false;
                }
            }
        }
         
    }
}


#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!(("\n\nPanic occurred: {}\nBye!", info.message()), fg: Color::LightRed);
    loop {}
}


#[no_mangle]
pub extern "C" fn rust_main() {
    vga_buffer::clear_screen();
    let mut last_scancode: u8 = 0;
    print!(
        ("\nWelcome to Mini Rust OS 1.0\nTo get a list of commands, type \"help\"\n"
    )
    );
    unsafe { print!(("\n{} >> ", USER_NAME)); }

    loop {
        let scancode = unsafe { read_keyboard_input() };

        if scancode != last_scancode {
            last_scancode = scancode;

            if scancode != 0 {
                handle_input(scancode);
            }
        }
    }
}