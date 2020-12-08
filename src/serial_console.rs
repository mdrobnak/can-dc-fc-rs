//#![deny(warnings)]
use crate::types::*;
use crate::{uprint, uprintln};
use core::fmt::Write;

pub fn display(
    tx: &mut SerialConsoleOutput,
    cd_state: &mut CDState,
    sys_ticks: u32,
    hundred_ms_counter: u8,
) {
    let verbose_console = cd_state.verbose_stats;
    let print_header = hundred_ms_counter % 250 == 0 || cd_state.quiet_to_verbose;
    let print_menu = cd_state.print_menu_request;

    if verbose_console {
        if print_header {
            print_header_to_serial(tx, verbose_console);
        }

        let mut line = 10;
        for i in cd_state.activity_list.iter() {
            uprintln!(tx, "\x1B[{};3H{}", line, i);
            line = line + 1;
        }

        if hundred_ms_counter % 5 == 0 {
            uprintln!(
                tx,
                "\x1B[20HPhase 1: {}\x1B[20;20HPhase 2: {}",
                cd_state.charger_relay_enabled,
                cd_state.latch_enabled,
            );
        }
        uprint!(
            tx,
            "\x1B[21HUptime: {}\x1B[21;20HCharge Type: ??",
            sys_ticks,
        ); // 18 characters
        uprint!(tx, "\x1B[22HState: {} \x1B[0K", cd_state.charge_state); // State is likely no more than 20 chars.
    } else if hundred_ms_counter % 5 == 0 {
        if print_menu {
            print_header_to_serial(tx, verbose_console);
        } else if print_header {
            uprintln!(
                tx,
                "Press v to enable verbose statistics. Press m for a list of commands."
            );
        }
        uprint!(tx, "State: {}  Charging: ", cd_state.charge_state); // State is likely no more than 20 chars.
        if cd_state.charger_relay_enabled {
            uprint!(tx, "Enabled   ");
        } else {
            uprint!(tx, "Disabled  ");
        }
        uprintln!(tx, "Uptime: {}", sys_ticks);
    }
}
pub fn print_header_to_serial(tx: &mut SerialConsoleOutput, verbose_console: bool) {
    if verbose_console {
        uprintln!(tx, "\x1B[2J\x1B[HCommands: ");
    } else {
        uprintln!(tx, "Commands: ");
    }
    uprintln!(tx, "c / C - Start / End Charge.");
    uprintln!(tx, "e - Clear / rEfresh the screen.");
    uprintln!(tx, "m - Show menu with verbose disabled.");
    uprintln!(tx, "v / V - Enable / Disable verbose.");
    if verbose_console {
        verbose_footer(tx);
    }
}
#[rustfmt::skip]
pub fn verbose_footer(tx: &mut SerialConsoleOutput) {
    uprintln!(tx, "Command? ");
    uprintln!(tx, "");
    uprintln!(tx, "                          Activity");
    uprintln!(tx, "+--------------------------------------------------------------+");
    uprintln!(tx, "|                                                              |");
    uprintln!(tx, "|                                                              |");
    uprintln!(tx, "|                                                              |");
    uprintln!(tx, "|                                                              |");
    uprintln!(tx, "|                                                              |");
    uprintln!(tx, "|                                                              |");
    uprintln!(tx, "|                                                              |");
    uprintln!(tx, "|                                                              |");
    uprintln!(tx, "+--------------------------------------------------------------+");
}
