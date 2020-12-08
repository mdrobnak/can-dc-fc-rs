#![deny(warnings)]
use crate::types::*;

pub fn init(mut hundred_ms_counter: u8, cd_state: &mut CDState, fc_can: &FCCAN) -> u8 {
    if cd_state.enable_can_transmit {
        params108(fc_can);
        status109(fc_can, cd_state);
    }
    if hundred_ms_counter < 255 {
        hundred_ms_counter = hundred_ms_counter + 1;
    } else {
        hundred_ms_counter = 0;
    }

    hundred_ms_counter
}

pub fn params108(fc_can: &FCCAN) {
    let id: u16 = 0x108;
    let size: u8 = 8;
    let mut params108_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    params108_frame.set_data_length(size.into());
    let params108 = params108_frame.data_as_mut();
    params108[0] = 0x00; // Weld check not supported.
    params108[1] = 174; // 60 frame.data.byte[1] + frame.data.byte[2] * 256;
    params108[2] = 1; // 1AE -> 430
    params108[3] = 0x20; // 32 Amps current available (not really...)
    params108[4] = 50; // Thresold frame.data.byte[4] + frame.data.byte[5] * 256;
    params108[5] = 0x00;
    params108[6] = 0x00;
    params108[7] = 0x00;
    fc_can.transmit(&params108_frame.into()).ok();
}

pub fn status109(fc_can: &FCCAN, cd_state: &CDState) {
    let id: u16 = 0x109;
    let size: u8 = 8;
    let mut status109_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    status109_frame.set_data_length(size.into());
    let status109 = status109_frame.data_as_mut();
    /* STATUS
     * define EVSE_STATUS_CHARGE		1 //charger is active
     * #define EVSE_STATUS_ERR			2 //something went wrong
     * #define EVSE_STATUS_CONNLOCK	4 //connector is currently locked
     * #define EVSE_STATUS_INCOMPAT	8 //params btwn vehicle and charger not compatible
     * #define EVSE_STATUS_BATTERR		16 //something wrong with battery?!
     * #define EVSE_STATUS_STOPPED		32 //charger is stopped
     */
    status109[0] = 0x02; // Protocol > 1 = 1.0
    status109[1] = 0x00; // Present voltage byte1 + 256*byte2
    status109[2] = 0x00;
    status109[3] = 0x00; // Current
    status109[4] = 0x00; // Reserved?
    status109[5] = 0x20; // Status
    if cd_state.latch_enabled {
        status109[5] += 4;
    }
    if cd_state.start_charge {
        status109[5] -= 32;
        status109[5] += 1;
    }
    status109[6] = 0xFF; // If < 0xFF then chgSecondsRemain = byte6 * 10;
    status109[7] = 0x10; // else chgSecondsRemain = byte7 * 60;
    fc_can.transmit(&status109_frame.into()).ok();
}
