#![deny(warnings)]
#[cfg(feature = "nucleof767zi")]
extern crate stm32f7xx_hal as hal;

#[cfg(any(feature = "nucleof446re",))]
extern crate stm32f4xx_hal as hal;

use arraydeque::{ArrayDeque, Wrapping};
use core::fmt::Display;
use heapless::consts::*;
use heapless::String;

impl Display for ChargeStateEnum {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::result::Result<(), core::fmt::Error> {
        match self {
            ChargeStateEnum::TimeOut => write!(f, "Timeout"),
            ChargeStateEnum::ChargeIdle => write!(f, "Idle"),
            ChargeStateEnum::InitiateCharge => write!(f, "Initiate Charge"),
            ChargeStateEnum::WaitForComms => write!(f, "Wait Comms"),
            ChargeStateEnum::WaitChargeEnable => write!(f, "Wait for Vehicle Enable"),
            ChargeStateEnum::ContactorRequest => write!(f, "Contactor Request"),
            ChargeStateEnum::ContactorFixed => write!(f, "Charge Enabled"),
            ChargeStateEnum::StopCharge => write!(f, "Stop Charge"),
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum ChargeStateEnum {
    TimeOut,
    ChargeIdle,
    InitiateCharge,
    WaitForComms,
    WaitChargeEnable,
    ContactorRequest,
    ContactorFixed,
    StopCharge,
}

pub struct CDState {
    pub activity_list: ArrayDeque<[String<U60>; 4], Wrapping>,
    pub charger_relay_enabled: bool,
    pub charge_state: ChargeStateEnum,
    pub comm_timeout: bool,
    pub enable_can_transmit: bool,
    pub evse_request: bool,
    pub latch_enabled: bool,
    pub previous_can_ts: u32,
    pub print_menu_request: bool,
    pub quiet_to_verbose: bool,
    pub start_charge: bool,
    pub verbose_stats: bool,
}

impl CDState {
    pub fn new() -> Self {
        // we create a method to instantiate `Foo`
        Self {
            activity_list: ArrayDeque::new(),
            charger_relay_enabled: false,
            charge_state: ChargeStateEnum::StopCharge,
            enable_can_transmit: false,
            comm_timeout: true,
            evse_request: false,
            latch_enabled: false,
            print_menu_request: false,
            previous_can_ts: 0,
            quiet_to_verbose: false,
            start_charge: false,
            verbose_stats: false,
        }
    }
}

// Generic type abstractions
// Why? Remove reference to hal, so that it does not need to be included in many spots with
// conditional code around it.
pub type BaseID = hal::can::BaseID;
pub type CanFrame = hal::can::CanFrame;
pub type DataFrame = hal::can::DataFrame;
pub type ID = hal::can::ID;
pub type Rtc = hal::rtc::Rtc;

// HW specific type abstractions
#[cfg(feature = "nucleof767zi")]
mod abstractions {
    extern crate stm32f7xx_hal as hal;
    use hal::can::Can;
    use hal::gpio::gpiod::{PD0, PD1, PD2};
    use hal::gpio::gpiog::{PG2, PG3};
    use hal::gpio::AF9;
    use hal::gpio::{Alternate, Floating, Input, Output, PushPull};
    use hal::pac::CAN1;
    pub type FCCAN = Can<CAN1, (PD1<Alternate<AF9>>, PD0<Alternate<AF9>>)>;
    pub type SerialConsoleOutput = hal::serial::Tx<hal::pac::USART3>;
    pub type FaultLinePin = PG2<Input<Floating>>;
    pub type RelayOnePin = PG3<Output<PushPull>>;
    pub type RelayTwoPin = PD2<Output<PushPull>>;
}

#[cfg(feature = "nucleof446re")]
mod abstractions {
    extern crate stm32f4xx_hal as hal;
    use hal::can::Can;
    use hal::gpio::gpiob::{PB3, PB5, PB6, PB8, PB9};
    use hal::gpio::AF9;
    use hal::gpio::{Alternate, Floating, Input, Output, PushPull};
    use hal::pac::CAN1;
    pub type FCCAN = Can<CAN1, (PB9<Alternate<AF9>>, PB8<Alternate<AF9>>)>;
    pub type SerialConsoleOutput = hal::serial::Tx<hal::pac::USART2>;
    pub type FaultLinePin = PB3<Input<Floating>>;
    pub type RelayOnePin = PB5<Output<PushPull>>;
    pub type RelayTwoPin = PB6<Output<PushPull>>; // FIXME: Not actual pin.
}

pub type FCCAN = abstractions::FCCAN;
pub type SerialConsoleOutput = abstractions::SerialConsoleOutput;
pub type FaultLinePin = abstractions::FaultLinePin;
pub type RelayOnePin = abstractions::RelayOnePin;
pub type RelayTwoPin = abstractions::RelayTwoPin;
