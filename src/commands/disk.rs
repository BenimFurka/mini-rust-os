use crate::vga_buffer::Color;
use crate::print;
use crate::drivers::ata::{AtaController, AtaDevice};

pub fn handle_disk_command(args: &[&str]) {
    if args.is_empty() || args[0] == "help" {
        disk_info();
        print!(("\n"));
        print!(("\nFor help use disk --help"), fg: Color::LightGray);
        return;
    }

    match args[0] {
        "info" => disk_info(),
        "read" => {
            if args.len() < 2 {
                print!(("\nError: missing sector number"), fg: Color::Red);
                print_help();
                return;
            }
            
            let sector = match parse_hex_u32(args[1]) {
                Some(addr) => addr,
                None => {
                    print!(("\nError: invalid sector number (use hex with 0x prefix)"), fg: Color::Red);
                    return;
                }
            };
            
            let count = if args.len() > 2 {
                match parse_hex_u32(args[2]) {
                    Some(n) if n > 0 && n <= 256 => n as u8,
                    _ => {
                        print!(("\nError: invalid sector count (1-256 in hex with 0x prefix)"), fg: Color::Red);
                        return;
                    }
                }
            } else { 1 };
            
            read_sectors(sector, count);
        },
        _ => print!(("\nUnknown disk command. Type 'disk help' for usage."), fg: Color::Red),
    }
}

fn print_help() {
    print!(("\nDisk commands:"), fg: Color::LightBlue);
    print!(("\n  disk info                - Show disk information"), fg: Color::White);
    print!(("\n  disk read <sector> [count] - Read sectors from disk (hex values with 0x)"), fg: Color::White);
    print!(("\n  disk --help               - Show this help"), fg: Color::White);
}

fn disk_info() {
    print!(("\nATA Devices:"), fg: Color::LightBlue);
    
    let devices = [
        ("Primary Master", AtaDevice::Primary),
        // TODO: Потестить это, лол
        //       ненавижу работать с дисками
        //("Primary Slave", AtaDevice::PrimarySlave),
        //("Secondary Master", AtaDevice::Secondary),
        //("Secondary Slave", AtaDevice::SecondarySlave),
    ];

    for (name, device) in &devices {
        print!(("\n  {}: ", name), fg: Color::LightBlue);
        
        let controller = AtaController::new(*device);
        
        match controller.identify() {
            Ok(identify_data) => {
                let mut model = [0u8; 40];
                for i in 0..20 {
                    let word = identify_data[27 + i];
                    model[i*2] = (word >> 8) as u8;
                    model[i*2 + 1] = (word & 0xFF) as u8;
                }
                let model_str = core::str::from_utf8(&model).unwrap_or("<invalid model>");
                print!(("{}", model_str.trim()), fg: Color::White);
                
                let sectors = identify_data[60] as u32 | ((identify_data[61] as u32) << 16);
                let capacity_gb = (sectors as f64) * 512.0 / (1024.0 * 1024.0 * 1024.0);
                print!(("\n  Capacity: "), fg: Color::LightBlue);
                print!(("{} sectors ({:.2} GB)", sectors, capacity_gb), fg: Color::White);

                let lba_support = (identify_data[49] & 0x200) != 0;
                let dma_support = (identify_data[49] & 0x100) != 0;
                print!(("\n  Features: "), fg: Color::LightBlue);
                print!(("{}{}",
                    if lba_support { "LBA " } else { "" },
                    if dma_support { "DMA" } else { "PIO" }), fg: Color::Green)
            }
            Err(e) => {
                print!(("{}", e));
            }
        }
    }
}

fn read_sectors(sector: u32, count: u8) {
    if count == 0 || count > 8 {
        print!(("\nError: Count must be between 1 and 8"), fg: Color::Red);
        return;
    }
    
    print!(("\nReading {} sector(s) from LBA 0x{:X}", count, sector));
    
    let devices = [
        ("Primary Master", AtaDevice::Primary),
        ("Primary Slave", AtaDevice::PrimarySlave),
        ("Secondary Master", AtaDevice::Secondary),
        ("Secondary Slave", AtaDevice::SecondarySlave),
    ];
    
    let mut found = false;
    
    for (name, device) in &devices {
        let controller = AtaController::new(*device);
        let mut buffer = [0u8; 512 * 8];
        
        print!(("\n  Trying {}... ", name));
        
        match controller.read_sectors(sector, count, &mut buffer[..(512 * count as usize)]) {
            Ok(()) => {
                print!(("OK"));
                found = true;
                
                let display_count = 64.min(512 * count as usize);
                print!(("\nFirst {} bytes of first sector:", display_count));
                
                for i in 0..display_count {
                    if i % 16 == 0 {
                        if i > 0 { 
                            print!(("\n  "));
                            for b in i-16..i {
                                let c = buffer.get(b).copied().unwrap_or(0);
                                print!(("{}", if c >= 32 && c < 127 { c as char } else { '.' }));
                            }
                        }
                        print!(("{:04X}: ", i));
                    }
                    print!(("{:02X} ", buffer[i]));
                }
                
                let remaining = display_count % 16;
                if remaining > 0 {
                    for _ in 0..(16 - remaining) * 3 { print!((" ",)) }
                    print!(("\n  "));
                    for b in display_count - remaining..display_count {
                        let c = buffer.get(b).copied().unwrap_or(0);
                        print!(("{}", if c >= 32 && c < 127 { c as char } else { '.' }));
                    }
                }
                
                break;
            }
            Err(e) => {
                print!(("\n{} (trying next device...)", e));
            }
        }
    }
    
    if !found {
        print!(("\nError: Could not read from any ATA device"), fg: Color::Red);
    }
}

fn parse_hex_u32(s: &str) -> Option<u32> {
    let s = s.trim_start_matches("0x");
    u32::from_str_radix(s, 16).ok()
}
