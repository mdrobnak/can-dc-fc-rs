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
            ChargeStateEnum::InsulationTest => write!(f, "Insulation Test"),
            ChargeStateEnum::WaitVehicleChargeStart => write!(f, "Wait for Vehicle Charge Start"),
            ChargeStateEnum::ChargeLoop => write!(f, "Charge Loop"),
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
    InsulationTest,
    WaitVehicleChargeStart,
    ChargeLoop,
    StopCharge,
}

pub struct CDState {
    pub activity_list: ArrayDeque<[String<U60>; 8], Wrapping>,
    pub charge_state: ChargeStateEnum,
    pub comm_timeout: bool,
    pub current_voltage: u16,
    pub delaycount: u8,
    pub enable_can_transmit: bool,
    pub evse_request: bool,
    pub latch_enabled: bool,
    pub previous_can_ts: u32,
    pub print_menu_request: bool,
    pub quiet_to_verbose: bool,
    pub simulate_insulation_test: bool,
    pub start_charge: bool,
    pub switch_one: bool,
    pub switch_two: bool,
    pub verbose_stats: bool,
}

impl CDState {
    pub fn new() -> Self {
        Self {
            activity_list: ArrayDeque::new(),
            charge_state: ChargeStateEnum::StopCharge,
            comm_timeout: true,
            current_voltage: 0,
            delaycount: 0,
            enable_can_transmit: false,
            evse_request: false,
            latch_enabled: false,
            previous_can_ts: 0,
            print_menu_request: false,
            quiet_to_verbose: false,
            simulate_insulation_test: false,
            start_charge: false,
            switch_one: false,
            switch_two: false,
            verbose_stats: false,
        }
    }
}

pub struct CarState {
    pub battery_over_temperature: bool,
    pub battery_over_voltage: bool,
    pub battery_pack_size: f32,
    pub battery_max_voltage: f32,
    pub battery_under_voltage: bool,
    pub charge_stop_request: bool,
    pub charge_time_estimate: f32,
    pub charge_time_max: f32,
    pub charging_enabled: bool,
    pub charging_malfunction: bool,
    pub contactor_open: bool,
    pub current_deviation: bool,
    pub current_target: u8,
    pub malfunction: bool,
    pub not_park: bool,
    pub stop_before_charge: bool,
    pub voltage_deviation: bool,
    pub voltage_target: u16,
    pub vehicle_parked: bool,
}

impl CarState {
    pub fn new() -> Self {
        Self {
            battery_over_temperature: false,
            battery_over_voltage: false,
            battery_pack_size: 0.0,
            battery_max_voltage: 0.0,
            battery_under_voltage: false,
            charge_stop_request: false,
            charge_time_estimate: 0.0,
            charge_time_max: 0.0,
            charging_enabled: false,
            charging_malfunction: false,
            contactor_open: true,
            current_deviation: false,
            current_target: 0,
            malfunction: false,
            not_park: true,
            stop_before_charge: false,

            voltage_deviation: false,
            voltage_target: 0,
            vehicle_parked: false,
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
