use crate::arch::x86_64::port::{Port, ReadOnly, ReadWrite, WriteOnly};
use crate::util::sync::SpinMutex;
use lazy_static::lazy_static;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum LineStatusFlags {
    InputFull = 1,
    OutputEmpty = 1 << 5,
}

pub struct Uart16550 {
    data: Port<u8, ReadWrite>,
    interrupts_enabled: Port<u8, WriteOnly>,
    fifo_control: Port<u8, WriteOnly>,
    line_control: Port<u8, WriteOnly>,
    modem_ctrl: Port<u8, WriteOnly>,
    line_status: Port<u8, ReadOnly>,
}

impl core::fmt::Write for Uart16550 {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.write(byte)
        }
        Ok(())
    }
}

impl Uart16550 {
    pub const unsafe fn new(base: u16) -> Self {
        Self {
            data: Port::read_write(base),
            interrupts_enabled: Port::write_only(base + 1),
            fifo_control: Port::write_only(base + 2),
            line_control: Port::write_only(base + 3),
            modem_ctrl: Port::write_only(base + 4),
            line_status: Port::read_only(base + 5),
        }
    }

    fn init(&mut self) {
        unsafe {
            // Disable interrupts
            self.interrupts_enabled.write(0x00);

            // Enable DLAB
            self.line_control.write(0x80);

            // Set maximum speed to 38400 bps by configuring DLL and DLM
            self.data.write(0x03);
            self.interrupts_enabled.write(0x00);

            // Disable DLAB and set data word length to 8 bits
            self.line_control.write(0x03);

            // Enable FIFO, clear TX/RX queues and
            // set interrupt watermark at 14 bytes
            self.fifo_control.write(0xC7);

            // Mark data terminal ready, signal request to send
            // and enable auxilliary output #2 (used as interrupt line for CPU)
            self.modem_ctrl.write(0x0B);

            // Enable  interrupts
            self.interrupts_enabled.write(0x01);
        }
    }

    #[allow(dead_code)]
    fn receive_available(&mut self) -> bool {
        unsafe { self.line_status.read() & LineStatusFlags::InputFull as u8 != 0 }
    }

    #[allow(dead_code)]
    fn receive(&mut self) -> u8 {
        unsafe {
            self.wait_for(LineStatusFlags::InputFull);
            self.data.read()
        }
    }

    #[allow(dead_code)]
    fn write_available(&mut self) -> bool {
        unsafe { self.line_status.read() & LineStatusFlags::OutputEmpty as u8 != 0 }
    }

    fn write(&mut self, byte: u8) {
        unsafe {
            let data = self.data.read();

            match data {
                8 | 0x7F => {
                    self.wait_for(LineStatusFlags::OutputEmpty);
                    self.data.write(8);

                    self.wait_for(LineStatusFlags::OutputEmpty);
                    self.data.write(b' ');

                    self.wait_for(LineStatusFlags::OutputEmpty);
                    self.data.write(8);
                }
                _ => {
                    self.wait_for(LineStatusFlags::OutputEmpty);
                    self.data.write(byte);
                }
            }
        }
    }

    unsafe fn wait_for(&mut self, status_flag: LineStatusFlags) {
        while self.line_status.read() & status_flag as u8 == 0 {
            core::hint::spin_loop();
        }
    }
}

lazy_static! {
    pub static ref SERIAL: SpinMutex<Uart16550> = {
        let mut serial = SpinMutex::new(unsafe { Uart16550::new(0x3F8) });
        serial.get_mut().init();
        serial
    };
}
