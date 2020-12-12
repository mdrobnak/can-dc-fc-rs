#![deny(warnings)]
use crate::types::*;

pub fn reset_car_data(mut car_state: &mut CarState) {
    car_state.battery_max_voltage = 0.0;
    car_state.battery_pack_size = 0.0;
    car_state.voltage_target = 0;
    car_state.charging_enabled = false;
    car_state.contactor_open = true;
}
