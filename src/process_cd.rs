#![deny(warnings)]
use crate::add_to_activity_list;
use crate::types::*;
use crate::utils::stop_charge;

// Logging
use heapless::consts::U60;
use heapless::String;
use ufmt::uwrite;

pub fn update_car_data(id: u32, data: &[u8], mut car_state: &mut CarState) {
    if id == 0x100 {
        // Battery ID
        car_state.battery_max_voltage = ((data[5] as u32) << 8 | data[4] as u32) as f32;
        car_state.battery_pack_size = data[6] as f32;
    }
    if id == 0x102 {
        car_state.current_target = data[3];
        car_state.voltage_target = ((data[2] as u16) << 8 | data[1] as u16) as u16;
        if (data[5] & 0x1) == 1 {
            car_state.charging_enabled = true;
        } else {
            car_state.charging_enabled = false;
        }
        if (data[5] & 0x2) == 2 {
            car_state.not_park = true;
        } else {
            car_state.not_park = false;
        }
        if (data[5] & 0x4) == 4 {
            car_state.malfunction = true;
        } else {
            car_state.malfunction = false;
        }
        if (data[5] & 0x8) == 8 {
            // open when 1
            car_state.contactor_open = true;
        } else {
            // closed when 0
            car_state.contactor_open = false;
        }
        if (data[5] & 0x10) == 0x10 {
            car_state.stop_before_charge = true;
        } else {
            car_state.stop_before_charge = false;
        }
    }
}
pub fn init(
    elapsed: u32,
    mut cd_state: &mut CDState,
    mut car_state: &mut CarState,
    id: u32,
    data: &[u8],
) {
    // Always attempt to update car data.
    update_car_data(id, &data, &mut car_state);
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
            if car_state.charging_enabled {
                add_to_activity_list!(cd_state, "{} - WaitChargeEnable -> InsulationTest", elapsed);
                cd_state.latch_enabled = true;
                cd_state.charge_state = ChargeStateEnum::InsulationTest;
            }
        }
        ChargeStateEnum::InsulationTest => {
            if cd_state.delaycount > 80 {
                add_to_activity_list!(
                    cd_state,
                    "{} - InsulationTest -> WaitVehicleChargeStart",
                    elapsed
                );
                cd_state.charge_state = ChargeStateEnum::WaitVehicleChargeStart;
                cd_state.delaycount = 0;
                cd_state.current_voltage = 0;
                cd_state.switch_two = true;
            } else {
                if cd_state.simulate_insulation_test {
                    if cd_state.current_voltage + 10 <= car_state.voltage_target {
                        cd_state.current_voltage += 10;
                    } else {
                        cd_state.current_voltage = car_state.voltage_target;
                    }
                }

                cd_state.delaycount += 1;
            }
        }
        ChargeStateEnum::WaitVehicleChargeStart => {
            if car_state.contactor_open == false && car_state.current_target > 0 {
                // Current > 0, Contactors closed.
                cd_state.start_charge = true;
                cd_state.charge_state = ChargeStateEnum::ChargeLoop;
                add_to_activity_list!(
                    cd_state,
                    "{} - WaitVehicleChargeStart -> ChargeLoop",
                    elapsed
                );
            }
            if car_state.charging_enabled == false {
                cd_state.charge_state = ChargeStateEnum::StopCharge;
                add_to_activity_list!(
                    cd_state,
                    "{} - WtVehChgSt -> StopCharge (Chg Disbld)",
                    elapsed
                );
            }
            if car_state.malfunction {
                cd_state.charge_state = ChargeStateEnum::StopCharge;
                add_to_activity_list!(cd_state, "{} - WtVehChgSt -> StopCharge (Malfunc)", elapsed);
            }
        }
        ChargeStateEnum::ChargeLoop => {
            if car_state.charging_enabled == false && car_state.current_target == 0 {
                cd_state.charge_state = ChargeStateEnum::StopCharge;
                add_to_activity_list!(cd_state, "{} - ChgLp -> StopCharge (Chg Disbld)", elapsed);
            }
            if car_state.malfunction {
                cd_state.charge_state = ChargeStateEnum::StopCharge;
                add_to_activity_list!(cd_state, "{} - ChgLp -> StopCharge (Malfnctn)", elapsed);
            }
        }
        ChargeStateEnum::StopCharge => {
            stop_charge(cd_state, car_state, elapsed);
        }
        ChargeStateEnum::TimeOut => {}
    }
}
