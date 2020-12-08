#![deny(warnings)]
use crate::add_to_activity_list;
use crate::types::*;

// Logging
use heapless::consts::U60;
use heapless::String;
use ufmt::uwrite;

pub fn init(elapsed: u32, mut cd_state: &mut CDState, id: u32, data: &[u8]) {
    // Main state machine for charge state here
    match cd_state.charge_state {
        ChargeStateEnum::ChargeIdle => {
            // Nothing to see here.
        }
        ChargeStateEnum::InitiateCharge => {
            // Nothing to see here as nothing coming in yet.
        }
        ChargeStateEnum::WaitForComms => {
            // Wait for 100,101,102 from EV
            match id {
                0x100 | 0x101 | 0x102 => {
                    // Compute max time...ehhh.
                    // Start transmitting 0x108, 0x109
                    cd_state.enable_can_transmit = true;
                    cd_state.charge_state = ChargeStateEnum::WaitChargeEnable;
                    add_to_activity_list!(
                        cd_state,
                        "{} - WaitForComms -> WaitChargeEnable",
                        elapsed
                    );
                }
                _ => {
                    // Do nothing.
                }
            }
        }
        ChargeStateEnum::WaitChargeEnable => {
            if id == 0x102 && (data[5] & 0x8) == 0 {
                cd_state.latch_enabled = true;
            }
            if id == 0x102 && (data[3] > 0) {
                cd_state.start_charge = true;
                cd_state.charge_state = ChargeStateEnum::ChargeLoop;
            }
        }
        ChargeStateEnum::ChargeLoop => {}
        ChargeStateEnum::StopCharge => {}
        ChargeStateEnum::TimeOut => {}
    }
}
