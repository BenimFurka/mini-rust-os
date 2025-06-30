use crate::spin::SpinMutex;
use x86_64::instructions::port::Port;

const PIC1_CMD: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;

const ICW4_8086: u8 = 0x01;

const PIC_EOI: u8 = 0x20;

const IRQ_OFFSET: u8 = 0x20;

pub struct Pic {
    offset: u8,
    command: Port<u8>,
    data: Port<u8>,
}

impl Pic {
    pub const fn new(offset: u8, command_port: u16, data_port: u16) -> Self {
        Self {
            offset,
            command: Port::new(command_port),
            data: Port::new(data_port),
        }
    }

    pub fn end_of_interrupt(&mut self) {
        unsafe { self.command.write(PIC_EOI); }
    }

    pub fn read_mask(&mut self) -> u8 {
        unsafe { self.data.read() }
    }

    pub fn write_mask(&mut self, mask: u8) {
        unsafe { self.data.write(mask); }
    }
}

pub struct ChainedPics {
    pub master: Pic,
    pub slave: Pic,
}

impl ChainedPics {
    pub const fn new() -> Self {
        Self {
            master: Pic::new(IRQ_OFFSET, PIC1_CMD, PIC1_DATA),
            slave: Pic::new(IRQ_OFFSET + 8, PIC2_CMD, PIC2_DATA),
        }
    }

    pub unsafe fn init(&mut self) {
        let master_mask = self.master.read_mask();
        let slave_mask = self.slave.read_mask();

        self.master.command.write(ICW1_INIT | ICW1_ICW4);
        self.slave.command.write(ICW1_INIT | ICW1_ICW4);

        self.master.data.write(self.master.offset);
        self.slave.data.write(self.slave.offset);

        self.master.data.write(4);
        self.slave.data.write(2);

        self.master.data.write(ICW4_8086);
        self.slave.data.write(ICW4_8086);

        self.master.write_mask(master_mask);
        self.slave.write_mask(slave_mask);
    }

    pub fn test(&mut self) -> bool {
        let master_mask = self.master.read_mask();
        let slave_mask = self.slave.read_mask();
        
        self.master.write_mask(0xAA);
        self.slave.write_mask(0x55);
        
        let master_ok = self.master.read_mask() == 0xAA;
        let slave_ok = self.slave.read_mask() == 0x55;
        
        self.master.write_mask(master_mask);
        self.slave.write_mask(slave_mask);
        
        master_ok && slave_ok
    }

    pub fn notify_end_of_interrupt(&mut self, irq: u8) {
        if irq >= 8 {
            self.slave.end_of_interrupt();
        }
        self.master.end_of_interrupt();
    }
}

pub static PICS: SpinMutex<ChainedPics> = SpinMutex::new(ChainedPics::new());

pub fn get_masks() -> (u8, u8) {
    let mut pics = PICS.lock();
    (pics.master.read_mask(), pics.slave.read_mask())
}

pub fn set_master_mask(mask: u8) {
    let mut pics = PICS.lock();
    pics.master.write_mask(mask);
}

pub fn set_slave_mask(mask: u8) {
    let mut pics = PICS.lock();
    pics.slave.write_mask(mask);
}

pub fn send_eoi(irq: u8) {
    let mut pics = PICS.lock();
    pics.notify_end_of_interrupt(irq);
}

pub fn init() -> Result<(), &'static str> {
    let mut pics = PICS.lock();
    unsafe { pics.init(); }
    
    if !pics.test() {
        return Err("PIC self-test failed");
    }
    
    Ok(())
}

pub fn test() -> bool {
    PICS.lock().test()
}
