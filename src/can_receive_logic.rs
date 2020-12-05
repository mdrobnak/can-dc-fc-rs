#![deny(warnings)]
use crate::process_cd::init as process_cd;
use crate::types::*;

pub fn init(can_frame: &CanFrame, elapsed: u32, mut cd_state: &mut CDState) {
    if let CanFrame::DataFrame(ref frame) = can_frame {
        let id: u32 = frame.id().into();
        let data = frame.data();

        // Can only say you've gotten a frame, not
        // that you _haven't_ gotten a frame.
        // Timeout needs to be set somewhere else.
        match id {
            0x100 | 0x101 | 0x102 => {
                cd_state.previous_can_ts = elapsed;
                cd_state.comm_timeout = false;
                process_cd(elapsed, &mut cd_state, id, data);
            }
            _ => {}
        }
    }
}
