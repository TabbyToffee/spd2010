pub mod driver;
pub mod io;

pub use driver::reset;

use core::{cell::RefCell, fmt};

use critical_section::Mutex;
use embedded_hal::i2c::I2c;
use heapless::Vec;

const SPD2010_ADDR: u8 = 0x53;
const SPD2010_MAX_TOUCH_POINTS: usize = 10;

#[derive(Debug)]
pub enum Error<I2C: I2c> {
    I2C(I2C::Error),
    ClearInterruptFailed,
}

pub struct FirmwareInfo {
    dummy: u32,
    dver: u16,
    pid: u32,
    ic_name_l: u32,
    ic_name_h: u32,
}

impl fmt::Display for FirmwareInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SPD2010 - Dummy: {}, Version: {}, PID: {}, IC Name: {}-{}",
            self.dummy, self.dver, self.pid, self.ic_name_h, self.ic_name_l
        )
    }
}

// All touches and gesture info
#[derive(Default, Debug)]
pub struct TouchData {
    pub points: Vec<TouchPoint, SPD2010_MAX_TOUCH_POINTS>,
    pub touch_count: u8,
    pub gesture: u8,
    pub down: bool,
    pub up: bool,
    pub down_x: u16,
    pub down_y: u16,
    pub up_x: u16,
    pub up_y: u16,
}

// Single touch
#[derive(Default, Debug)]
pub struct TouchPoint {
    pub id: u8,
    pub x: u16,
    pub y: u16,
    pub weight: u8,
}

#[derive(Debug)]
struct StatusLow {
    pt_exist: bool,
    gesture: bool,
    key: bool,
    aux: bool,
    keep: bool,
    raw_or_pt: bool,
    none6: bool,
    none7: bool,
}

#[derive(Debug)]
struct StatusHigh {
    none0: bool,
    none1: bool,
    none2: bool,
    cpu_run: bool,
    tint_low: bool,
    tic_in_cpu: bool,
    tic_in_bios: bool,
    tic_busy: bool,
}

#[derive(Debug)]
struct TouchStatus {
    status_low: StatusLow,
    status_high: StatusHigh,
    read_len: u16,
}

#[derive(Default)]
struct HDPStatus {
    status: u8,
    next_packet_len: u16,
}

pub struct SPD2010Touch<'a, I2C: I2c, Ti: InterruptInput> {
    i2c: I2C,
    touch_interrupt: &'a Mutex<RefCell<Option<Ti>>>,
}

impl<'a, I2C: I2c, Ti: InterruptInput> SPD2010Touch<'a, I2C, Ti> {
    pub fn new(i2c: I2C, touch_interrupt: &'a Mutex<RefCell<Option<Ti>>>) -> Self {
        // touch_interrupt.listen(Event::FallingEdge);

        Self {
            i2c,
            touch_interrupt,
        }
    }

    fn clear_interrupt_flag(&self) {
        critical_section::with(|cs| {
            self.touch_interrupt
                .borrow_ref_mut(cs)
                .as_mut()
                .unwrap()
                .clear_interrupt_flag();
        });
    }

    fn get_interrupt_flag(&self) -> bool {
        critical_section::with(|cs| {
            self.touch_interrupt
                .borrow_ref(cs)
                .as_ref()
                .unwrap()
                .get_interrupt_flag()
        })
    }

    fn get_interrupt_state(&self) -> bool {
        critical_section::with(|cs| {
            self.touch_interrupt
                .borrow_ref(cs)
                .as_ref()
                .unwrap()
                .get_interrupt_state()
        })
    }
}

pub trait InterruptInput {
    fn get_interrupt_flag(&self) -> bool;
    fn clear_interrupt_flag(&mut self);
    fn get_interrupt_state(&self) -> bool;
}
