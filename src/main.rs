#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate cortex_m;
extern crate panic_halt;

// Entrypoint
use cortex_m_rt::entry;

#[cfg(feature = "nucleof767zi")]
extern crate stm32f7xx_hal as hal;

#[cfg(feature = "nucleof446re")]
extern crate stm32f4xx_hal as hal;

// General HAL items
use hal::{
    interrupt, pac,
    prelude::*,
    timer::{Event, Timer},
};

// Physical OK to charge signal...
// Used to clear the pending interrupt bit in the interrupt handler.
use hal::gpio::ExtiPin;

// Elapsed_MS stuff...
use core::cell::{Cell, RefCell};
use core::ops::DerefMut;
use cortex_m::interrupt::{free, Mutex};

// CAN
use hal::can::RxFifo;

// Aliases
use can_dc_fc::can_receive_logic::init as can_receive_logic;
use can_dc_fc::hundred_ms_loop::init as hundred_ms_loop;
use can_dc_fc::process_serial::init as process_serial;
use can_dc_fc::serial_console::display as serial_console;
use can_dc_fc::types::*;

// Random Rust notes:
// fn main() { needs to be fn main() -> ! to show it will never return.
// fn main() { << Standard exampel
// References to |cs| refer to critical (non-preemptible) sections

// Dragons ahead.
static ELAPSED_MS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0u32));
static TIMER_TIM2: Mutex<RefCell<Option<Timer<pac::TIM2>>>> = Mutex::new(RefCell::new(None));

// Another Semaphore / Mutex for use with the Fault Line input
static SEMAPHORE: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));
static FAULT_LINE: Mutex<RefCell<Option<FaultLinePin>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // Hardware to initialize:
    // Relay One Output
    // Relay Two Output
    // Fast Charge CAN Tx, Rx
    // Clocks
    // Serial port
    // RTC (No alarms yet)
    // TIM2 SysTick

    let (fault_in, mut relay_1, mut relay_2, fc_can, serial, timer, _rtc) =
        can_dc_fc::hardware_init::init_devices();

    // Interrupts / Mutexes
    free(|cs| {
        TIMER_TIM2.borrow(cs).replace(Some(timer));
    });
    free(|cs| {
        FAULT_LINE.borrow(cs).replace(Some(fault_in));
    });
    can_dc_fc::hardware_init::enable_interrupts();

    let (mut tx, mut rx) = serial.split();

    const HUNDRED_MS: u32 = 100;
    let mut previous_100_ms_ts = 0;
    let mut hundred_ms_counter: u8 = 0;

    // Create the status structure
    let mut cd_state = CDState::new();
    let mut car_state = CarState::new();
    // Status queue things
    // Too many of these items slows down serial console, which slows down
    // all of the loops.

    // Main control loop here.
    // Process serial input
    // Run X ms loops (10, 100, 1000)

    loop {
        let elapsed = free(|cs| ELAPSED_MS.borrow(cs).get());

        // Highly interactive pieces:
        // CAN reception
        for fifo in &[RxFifo::Fifo0, RxFifo::Fifo1] {
            if let Ok(rx_frame) = fc_can.receive(fifo) {
                can_receive_logic(&rx_frame, elapsed, &mut cd_state, &mut car_state);
            }
        }

        // Serial input (and some output) - BUT - only gets called when there is input!
        if let Ok(received) = rx.read() {
            process_serial(received, elapsed, &mut cd_state, &mut car_state);
        }

        // 10 ms - Done
        /*        if (elapsed - previous_10_ms_ts) >= TEN_MS {
            previous_10_ms_ts = elapsed;
        }
        */

        // 100 ms - Done
        if (elapsed - previous_100_ms_ts) >= HUNDRED_MS {
            previous_100_ms_ts = elapsed;
            hundred_ms_counter =
                hundred_ms_loop(hundred_ms_counter, &mut cd_state, &mut car_state, &fc_can);
            if cd_state.switch_one {
                relay_1.set_low().ok();
            } else {
                relay_1.set_high().ok();
            }
            if cd_state.switch_one && cd_state.switch_two {
                relay_2.set_low().ok();
            } else {
                relay_2.set_high().ok();
            }

            serial_console(
                &mut tx,
                &mut cd_state,
                &mut car_state,
                elapsed,
                hundred_ms_counter,
            );

            // Once run, flip it off.
            if cd_state.quiet_to_verbose {
                cd_state.quiet_to_verbose = false;
            }
            if cd_state.print_menu_request {
                cd_state.print_menu_request = false;
            }
        }
    }
}

#[interrupt]
fn TIM2() {
    free(|cs| {
        if let Some(ref mut tim2) = TIMER_TIM2.borrow(cs).borrow_mut().deref_mut() {
            tim2.clear_interrupt(Event::TimeOut);
        }

        let cell = ELAPSED_MS.borrow(cs);
        let val = cell.get();
        cell.replace(val + 1);
    });
}

#[cfg(feature = "nucleof767zi")]
#[interrupt]
fn EXTI2() {
    // This is going to fire for all pins associated with this interrupt, which is going to be all
    // of them ending in 2 - PA2,PB2,...PG2, etc. So avoid using any more pins with the end in 2
    // until it is known how to differentiate between them.
    // Answer: "using EXTI_PR you have to detect which pin generated interrupt"
    free(|cs| {
        match FAULT_LINE.borrow(cs).borrow_mut().as_mut() {
            // Clear the push button interrupt
            Some(b) => {
                b.clear_interrupt_pending_bit();
                if b.is_high().unwrap_or(false) {
                    SEMAPHORE.borrow(cs).set(true);
                } else {
                    SEMAPHORE.borrow(cs).set(false);
                }
            }

            // This should never happen
            None => (),
        }
    });
}

#[cfg(feature = "nucleof446re")]
#[interrupt]
fn EXTI3() {
    // This is going to fire for all pins associated with this interrupt, which is going to be all
    // of them ending in 3 - PA3,PB3,...PG2, etc. So avoid using any more pins with the end in 2
    // until it is known how to differentiate between them.
    // Answer: "using EXTI_PR you have to detect which pin generated interrupt"
    free(|cs| {
        match FAULT_LINE.borrow(cs).borrow_mut().as_mut() {
            // Clear the push button interrupt
            Some(b) => {
                b.clear_interrupt_pending_bit();
                if b.is_high().unwrap_or(false) {
                    SEMAPHORE.borrow(cs).set(true);
                } else {
                    SEMAPHORE.borrow(cs).set(false);
                }
            }

            // This should never happen
            None => (),
        }
    });
}
