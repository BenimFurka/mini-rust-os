use crate::drivers::pic;
use crate::vga_buffer::Color;
use crate::print;

pub fn handle_pic_command(args: &[&str]) {
    if args.is_empty() || args[0] == "--help" {
        print_help();
        return;
    }

    match args[0] {
        "test" => test_pic(),
        "mask" => handle_mask_command(&args[1..]),
        "eoi" => handle_eoi_command(&args[1..]),
        "status" => show_status(),
        _ => print!(("\nUnknown PIC command. Type 'pic --help' for usage."), fg: Color::Red),
    }
}

fn test_pic() {
    print!(("\nTesting PIC... "), fg: Color::White);
    if pic::test() {
        print!(("OK"), fg: Color::LightGreen);
    } else {
        print!(("FAILED"), fg: Color::Red);
    }
}

fn handle_mask_command(args: &[&str]) {
    if args.is_empty() {
        let (master_mask, slave_mask) = pic::get_masks();
        print!(("\nMaster PIC mask: 0x{:02X}", master_mask), fg: Color::LightCyan);
        print!(("\nSlave PIC mask: 0x{:02X}", slave_mask), fg: Color::LightCyan);
        return;
    }

    if args.len() < 2 {
        print!(("\nMissing value for mask command"), fg: Color::Red);
        print_help();
        return;
    }

    let pic_type = args[0];
    let value = match u8::from_str_radix(args[1].trim_start_matches("0x"), 16) {
        Ok(v) => v,
        Err(_) => {
            print!(("\nInvalid mask value. Use hex (e.g., 0x20)"), fg: Color::Red);
            return;
        }
    };

    match pic_type {
        "master" => {
            pic::set_master_mask(value);
            print!(("\nMaster PIC mask set to 0x{:02X}", value), fg: Color::LightGreen);
        }
        "slave" => {
            pic::set_slave_mask(value);
            print!(("\nSlave PIC mask set to 0x{:02X}", value), fg: Color::LightGreen);
        }
        _ => print!(("\nInvalid PIC type. Use 'master' or 'slave'"), fg: Color::Red),
    }
}

fn handle_eoi_command(args: &[&str]) {
    if args.is_empty() {
        print!(("\nMissing IRQ number for EOI command"), fg: Color::Red);
        print_help();
        return;
    }

    let irq = match args[0].parse::<u8>() {
        Ok(irq) if irq < 16 => irq,
        _ => {
            print!(("\nInvalid IRQ number. Must be 0-15"), fg: Color::Red);
            return;
        }
    };

    pic::send_eoi(irq);
    
    print!(("\nSent EOI for IRQ {}", irq), fg: Color::LightGreen);
}

fn show_status() {
    let (master_mask, slave_mask) = pic::get_masks();
    
    print!(("\nPIC Status:"), fg: Color::LightBlue);
    print!(("\n  Master PIC mask: 0x{:02X}", master_mask), fg: Color::White);
    print!(("\n  Slave PIC mask:  0x{:02X}", slave_mask), fg: Color::White);
    
    print!(("\nMasked IRQs (1 = masked, 0 = unmasked):"), fg: Color::LightBlue);
    print!(("\n  Master (0-7):  "), fg: Color::White);
    for i in 0..8 {
        print!(("{}", if master_mask & (1 << i) != 0 { '1' } else { '0' }));
    }
    print!(("\n  Slave (8-15):  "), fg: Color::White);
    for i in 0..8 {
        print!(("{}", if slave_mask & (1 << i) != 0 { '1' } else { '0' }));
    }
}

fn print_help() {
    print!(("\nPIC (Programmable Interrupt Controller) commands:"), fg: Color::LightBlue);
    print!(("\n  pic test                - Test PIC functionality"), fg: Color::White);
    print!(("\n  pic mask [master|slave] [value] - Get or set interrupt masks"), fg: Color::White);
    print!(("\n  pic eoi <irq>           - Send End of Interrupt for a specific IRQ"), fg: Color::White);
    print!(("\n  pic status              - Show PIC status"), fg: Color::White);
    print!(("\n  pic --help             - Show this help message"), fg: Color::White);
}
