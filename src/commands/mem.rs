use x86_64::VirtAddr;
use crate::vga_buffer::Color;
use crate::print;

pub fn handle_mem_command(args: &[&str]) {
    if args.is_empty() || args[0] == "--help" {
        print_help();
        return;
    }

    match args[0] {
        "read" => handle_mem_read(&args[1..]),
        "write" => handle_mem_write(&args[1..]),
        _ => print!(("\nUnknown memory command. Type 'mem --help' for usage."), fg: Color::Red),
    }
}

fn handle_mem_read(args: &[&str]) {
    if args.is_empty() {
        print!(("\nMissing memory address"), fg: Color::Red);
        print_help();
        return;
    }

    let addr = match parse_hex_u64(args[0]) {
        Some(addr) => addr,
        None => {
            print!(("\nInvalid address. Use hex (e.g., 0x1000)"), fg: Color::Red);
            return;
        }
    };

    let length = if args.len() > 1 {
        match args[1].parse::<usize>() {
            Ok(len) if len > 0 && len <= 64 => len,
            _ => 16, 
        }
    } else {
        16
    };

    let virt_addr = VirtAddr::new(addr);
    
    if !is_address_accessible(virt_addr) {
        print!(("\nCannot access memory at address 0x{:X}", addr), fg: Color::Red);
        return;
    }

    print!(("\nMemory at 0x{:016X}: ", addr), fg: Color::LightBlue);
    
    for i in 0..length {
        if i % 16 == 0 {
            if i > 0 {
                print!((" |"));
                for j in (i-16)..i {
                    if j < length {
                        let byte = unsafe { read_byte(virt_addr + j as u64) };
                        if byte.is_ascii_graphic() {
                            print!(("{}", byte as char), fg: Color::White);
                        } else {
                            print!(("."), fg: Color::DarkGray);
                        }
                    }
                }
                print!(("|"));
            }
            print!(("\n  {:016X}: ", addr + i as u64), fg: Color::LightCyan);
        }
        
        if i < length {
            let byte = unsafe { read_byte(virt_addr + i as u64) };
            print!(("{:02X} ", byte), fg: Color::White);
        }
    }
    
    let remaining = length % 16;
    if remaining > 0 {
        let padding = (16 - remaining) * 3;
        for _ in 0..padding { print!((" "), fg: Color::White); }
        print!((" |"), fg: Color::White);
        
        let start = length - remaining;
        for j in start..length {
            let byte = unsafe { read_byte(virt_addr + j as u64) };
            if byte.is_ascii_graphic() {
                print!(("{}", byte as char), fg: Color::White);
            } else {
                print!(("."), fg: Color::DarkGray);
            }
        }
        print!(("|"), fg: Color::White);
    }
}

fn handle_mem_write(args: &[&str]) {
    if args.len() < 2 {
        print!(("\nMissing address or value"), fg: Color::Red);
        print_help();
        return;
    }

    let addr = match parse_hex_u64(args[0]) {
        Some(addr) => addr,
        None => {
            print!(("\nInvalid address. Use hex (e.g., 0x1000)"), fg: Color::Red);
            return;
        }
    };

    let value = match parse_hex_u8(args[1]) {
        Some(val) => val,
        None => {
            print!(("\nInvalid value. Use hex (e.g., 0x41)"), fg: Color::Red);
            return;
        }
    };

    let virt_addr = VirtAddr::new(addr);
    
    if !is_address_writable(virt_addr) {
        print!(("\nCannot write to memory at address 0x{:X}", addr), fg: Color::Red);
        return;
    }
    
    unsafe {
        let ptr = virt_addr.as_mut_ptr::<u8>();
        let old_value = ptr.read_volatile();
        ptr.write_volatile(value);
        print!(("\nWrote 0x{:02X} to 0x{:016X} (was: 0x{:02X})", value, addr, old_value), 
              fg: Color::LightGreen);
    }
}

unsafe fn read_byte(addr: VirtAddr) -> u8 {
    let ptr = addr.as_ptr::<u8>();
    ptr.read_volatile()
}

fn is_address_accessible(addr: VirtAddr) -> bool {
    // TODO: Слишком базово я думаю
    !addr.is_null()
}

fn is_address_writable(addr: VirtAddr) -> bool {
    // TODO: Мне не нравится
    is_address_accessible(addr)
}

fn parse_hex_u64(s: &str) -> Option<u64> {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).ok()
}

fn parse_hex_u8(s: &str) -> Option<u8> {
    u8::from_str_radix(s.trim_start_matches("0x"), 16).ok()
}

fn print_help() {
    print!(("\nMemory commands:"), fg: Color::LightBlue);
    print!(("\n  mem read <address> [length] - Read memory at address (default: 16 bytes)"), fg: Color::White);
    print!(("\n  mem write <address> <value> - Write value to memory at address"), fg: Color::White);
    print!(("\n  mem --help                 - Show this help message"), fg: Color::White);
}
