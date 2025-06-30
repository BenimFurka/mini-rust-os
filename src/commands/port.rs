use x86_64::instructions::port::Port;
use crate::vga_buffer::Color;
use crate::print;

pub fn handle_port_command(args: &[&str]) {
    if args.is_empty() || args[0] == "--help" {
        print_help();
        return;
    }

    match args[0] {
        "read" => handle_port_read(&args[1..]),
        "write" => handle_port_write(&args[1..]),
        "list" => list_ports(),
        _ => print!(("\nUnknown port command. Type 'port --help' for usage."), fg: Color::Red),
    }
}

fn handle_port_read(args: &[&str]) {
    if args.is_empty() {
        print!(("\nMissing port address"), fg: Color::Red);
        print_help();
        return;
    }

    let port_addr = match parse_hex_u16(args[0]) {
        Some(addr) => addr,
        None => {
            print!(("\nInvalid port address. Use hex (e.g., 0x60)"), fg: Color::Red);
            return;
        }
    };

    // Safety: Reading from I/O ports is inherently unsafe
    let value = unsafe {
        let mut port = Port::<u8>::new(port_addr);
        port.read()
    };

    print!(("\nPort 0x{:04X}: 0x{:02X} ({} dec)", port_addr, value, value), fg: Color::LightGreen);
}

fn handle_port_write(args: &[&str]) {
    if args.len() < 2 {
        print!(("\nMissing port address or value"), fg: Color::Red);
        print_help();
        return;
    }

    let port_addr = match parse_hex_u16(args[0]) {
        Some(addr) => addr,
        None => {
            print!(("\nInvalid port address. Use hex (e.g., 0x60)"), fg: Color::Red);
            return;
        }
    };

    let value = match parse_hex_u8(args[1]) {
        Some(val) => val,
        None => {
            print!(("\nInvalid value. Use hex (e.g., 0x20)"), fg: Color::Red);
            return;
        }
    };

    // Safety: Writing to I/O ports is inherently unsafe
    unsafe {
        let mut port = Port::<u8>::new(port_addr);
        port.write(value);
    }

    print!(
        ("\nWrote 0x{:02X} to port 0x{:04X}", value, port_addr),
        fg: Color::LightGreen
    );
}

fn list_ports() {
    print!(("\nCommon I/O Ports:"), fg: Color::LightBlue);
    print!(("\n  0x20-0x21 - Master PIC"));
    print!(("\n  0xA0-0xA1 - Slave PIC"));
    print!(("\n  0x40-0x43 - PIT (Programmable Interval Timer)"));
    print!(("\n  0x60, 0x64 - Keyboard Controller"));
    print!(("\n  0x3F8-0x3FF - COM1 (Serial Port 1)"));
    print!(("\n  0x2F8-0x2FF - COM2 (Serial Port 2)"));
    print!(("\n  0x1F0-0x1F7 - Primary IDE Controller"));
    print!(("\n  0x170-0x177 - Secondary IDE Controller"));
    print!(("\n  0x3C0-0x3DF - VGA"));
}

fn parse_hex_u16(s: &str) -> Option<u16> {
    u16::from_str_radix(s.trim_start_matches("0x"), 16).ok()
}

fn parse_hex_u8(s: &str) -> Option<u8> {
    u8::from_str_radix(s.trim_start_matches("0x"), 16).ok()
}

fn print_help() {
    print!(("\nPort I/O commands:"), fg: Color::LightBlue);
    print!(("\n  port read <address>     - Read value from an I/O port"), fg: Color::White);
    print!(("\n  port write <address> <value> - Write value to an I/O port"), fg: Color::White);
    print!(("\n  port list              - List common I/O ports"), fg: Color::White);
    print!(("\n  port --help            - Show this help message"), fg: Color::White);
}
