#![deny(warnings)]
use crate::add_to_activity_list;
use crate::types::CDState;
use crate::types::*;
use crate::utils::stop_charge;

// Logging
use heapless::consts::U60;
use heapless::String;
use ufmt::uwrite;

pub fn init(command: u8, elapsed: u32, mut cd_state: &mut CDState, mut car_state: &mut CarState) {
    normal_input(command, elapsed, &mut cd_state, &mut car_state);
}
pub fn normal_input(
    command: u8,
    elapsed: u32,
    mut cd_state: &mut CDState,
    car_state: &mut CarState,
) {
    match command {
        // a
        0x61 => {}
        // A
        0x41 => {}
        // c
        0x63 => {
            add_to_activity_list!(cd_state, "{} - User initied start of charge.", elapsed);
            // Turn on Relay to power EV side.
            cd_state.switch_one = true;
            cd_state.charge_state = ChargeStateEnum::WaitForComms;
            add_to_activity_list!(cd_state, "{} - InitiateCharge -> WaitForComms", elapsed);
        }
        // C
        0x43 => {
            cd_state.charge_state = ChargeStateEnum::StopCharge;
            add_to_activity_list!(cd_state, "{} - User initied stop of charge.", elapsed);
            stop_charge(cd_state, car_state, elapsed);
        }
        // d
        0x64 => {}
        // D
        0x44 => {}
        // e
        0x65 => {
            cd_state.quiet_to_verbose = true;
        }

        // m
        0x6D => {
            cd_state.print_menu_request = true;
        }
        // r
        0x72 => {}
        // R
        0x52 => {}
        // s
        0x73 => {}
        // v
        0x76 => {
            cd_state.verbose_stats = true;
            cd_state.quiet_to_verbose = true;
        }
        // V
        0x56 => {
            cd_state.verbose_stats = false;
        }
        _ => {
            add_to_activity_list!(cd_state, "{} - Invalid selection!", elapsed);
        }
    }
}
