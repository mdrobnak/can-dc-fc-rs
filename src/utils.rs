#![deny(warnings)]
use crate::add_to_activity_list;
use crate::types::*;

// Logging
use heapless::consts::U60;
use heapless::String;
use ufmt::uwrite;

pub fn reset_car_data(mut car_state: &mut CarState) {
    car_state.battery_max_voltage = 0.0;
    car_state.battery_pack_size = 0.0;
    car_state.voltage_target = 0;
    car_state.charging_enabled = false;
    car_state.contactor_open = true;
}

pub fn stop_charge(mut cd_state: &mut CDState, mut car_state: &mut CarState, elapsed: u32) {
    reset_car_data(&mut car_state);
    cd_state.switch_one = false;
    cd_state.switch_two = false;
    cd_state.latch_enabled = false;
    cd_state.enable_can_transmit = false;
    cd_state.current_voltage = 0;
    cd_state.charge_state = ChargeStateEnum::ChargeIdle;
    add_to_activity_list!(cd_state, "{} - StopCharge -> ChargeIdle", elapsed);
}
