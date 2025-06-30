#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

extern crate rlibc;

mod allocator;
mod drivers;
mod port;
mod spin;
mod vga_buffer;
mod handle_command;
pub mod commands;

pub use port::{inb, outb, inw, outw, inl, outl};

use core::panic::PanicInfo;
use handle_command::handle_command;
use vga_buffer::{Color, WRITER};

pub use drivers::pic;
extern crate volatile;

extern "C" {
    fn read_keyboard_input() -> u8;
}

#[macro_export]
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

// TODO: Эта клавиатура с самого начала в основном файле
//       стоит попробовать перенести в drivers/ наверное
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
                    WRITER.lock().delete_char();
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

                print!(("\n>> "), fg: Color::LightGreen);
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
                        let char_to_print = if CAPS_LOCK_PRESSED {
                            character.to_ascii_uppercase()
                        } else {
                            character
                        };
                        print!(("{}", char_to_print), fg: Color::LightGray);
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


#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    vga_buffer::clear_screen();
    
    print!(("\nWelcome to Mini Rust OS 1.0\n"), fg: Color::LightBlue);
    
    print!(("Initializing PIC... "), fg: Color::White);
    match drivers::pic::init() {
        Ok(_) => print!(("OK\n"), fg: Color::LightGreen),
        Err(e) => {
            print!(("FAILED: {}\n", e), fg: Color::Red);
            print!(("Continuing with limited functionality\n"), fg: Color::Yellow);
        }
    }
    
    print!(("Testing PIC... "), fg: Color::White);
    if drivers::pic::test() {
        print!(("OK\n"), fg: Color::LightGreen);
    } else {
        print!(("FAILED\n"), fg: Color::Red);
    }
    
    print!(("\nType 'help' for a list of commands\n"), fg: Color::LightGray);
    print!(("\n>> "), fg: Color::LightGreen);
    
    let mut last_scancode: u8 = 0;

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

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;
    let _lock = vga_buffer::WRITER_LOCK.lock();
    let mut writer = vga_buffer::WRITER.lock();
    
    writer.set_color(vga_buffer::Color::LightRed, vga_buffer::Color::Black);
    let _ = write!(writer, "\n\nKernel panic: {}\n", info);
    loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}