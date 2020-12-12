#![deny(warnings)]
#[cfg(feature = "nucleof767zi")]
extern crate stm32f7xx_hal as hal;

#[cfg(feature = "nucleof446re")]
extern crate stm32f4xx_hal as hal;

#[cfg(feature = "nucleof767zi")]
use hal::serial::Config;

#[cfg(feature = "nucleof446re")]
use hal::serial::config::Config;

use cortex_m::peripheral::NVIC;
use hal::{
    interrupt, pac,
    prelude::*,
    serial::Serial,
    timer::{Event, Timer},
};

use hal::gpio::Alternate;
use hal::gpio::AF7;
use hal::gpio::{Edge, ExtiPin};
use hal::rtc::Rtc;

// CAN
use hal::can::Can;
use hal::can::CanBitTiming;
use hal::can::CanConfig;
use hal::can::CanFilterConfig;

use crate::types::*;

pub fn enable_interrupts() {
    unsafe {
        NVIC::unmask(pac::Interrupt::TIM2);
    }
    #[cfg(feature = "nucleof767zi")]
    unsafe {
        NVIC::unmask::<interrupt>(interrupt::EXTI2);
    }
    #[cfg(feature = "nucleof446re")]
    unsafe {
        NVIC::unmask::<interrupt>(interrupt::EXTI3);
    }
}

#[cfg(feature = "nucleof767zi")]
pub fn init_devices() -> (
    FaultLinePin,
    RelayOnePin,
    RelayTwoPin,
    FCCAN,
    hal::serial::Serial<
        hal::pac::USART3,
        (
            hal::gpio::gpiod::PD8<Alternate<AF7>>,
            hal::gpio::gpiod::PD9<Alternate<AF7>>,
        ),
    >,
    hal::timer::Timer<pac::TIM2>,
    hal::rtc::Rtc,
) {
    // Hardware to initialize:
    // Fault Input
    // Latch Output
    // CAN Tx, Rx
    // Serial port
    // TIM2
    let mut p = pac::Peripherals::take().unwrap();
    let mut syscfg = p.SYSCFG;
    let mut exti = p.EXTI;

    // GPIO D for CAN (PD0,1) and USART3 (PD8,9) on STM32F767
    let gpiod = p.GPIOD.split();

    // GPIO G for Fault and Latch I/O (PG2 for Fault (Read), and PG3 for Latch (Push-Pull High
    // output)).
    let gpiog = p.GPIOG.split();

    let mut fault_in = gpiog.pg2.into_floating_input();
    fault_in.make_interrupt_source(&mut syscfg, &mut p.RCC);
    let mut relay_1 = gpiog.pg3.into_push_pull_output();
    relay_1.set_high().ok();
    let mut relay_2 = gpiod.pd2.into_push_pull_output();
    relay_2.set_high().ok();

    // Set trigger and enable interrupt.
    fault_in.trigger_on_edge(&mut exti, Edge::RISING_FALLING);
    fault_in.enable_interrupt(&mut exti);

    // Freeze RCC and System Clocks *After* setting EXTI items.
    // Run both boards at 180 as we don't need the extra 36MHz speed.
    // Enable the HSE 8MHz crystal on both boards. F4 board is impossible to use without it.
    // F7 board is marginally better, but consistency is good.
    let mut rcc = p.RCC.constrain();
    let clocks = rcc
        .cfgr
        .hse(hal::rcc::HSEClock {
            freq: 8_000_000,
            mode: hal::rcc::HSEClockMode::Oscillator,
        })
        .sysclk(180.mhz())
        .freeze();

    // AF7 -> Alternate Function 7 -> USART for PD8/9.
    let tx_pin = gpiod.pd8.into_alternate_af7();
    let rx_pin = gpiod.pd9.into_alternate_af7();
    let serial = Serial::new(
        p.USART3,
        (tx_pin, rx_pin),
        clocks,
        Config {
            baud_rate: 230_400.bps(),
            oversampling: hal::serial::Oversampling::By16,
            character_match: None,
        },
    );

    // Timer
    let mut timer = Timer::tim2(p.TIM2, 1.khz(), clocks, &mut rcc.apb1);
    timer.listen(Event::TimeOut);

    // -- CAN BUS --
    // Set up CAN bit timing.
    // Bit Timing for 180MHz System Clock
    // (45MHz APB1)
    // CAN_BTR: 0x001e0004
    const BIT_TIMING: CanBitTiming = CanBitTiming {
        prescaler: 4, // Prescaler: 5
        sjw: 0,       // CAN_SJW_1TQ
        bs1: 14,      // CAN_BS1_15TQ
        bs2: 1,       // CAN_BS2_2TQ
    };

    pub const HV_CAN_CONFIG: CanConfig = CanConfig {
        loopback_mode: false,
        silent_mode: false,
        ttcm: false,
        abom: true,
        awum: false,
        nart: false,
        rflm: false,
        txfp: false,
        // TODO - update CAN impl to calculate these
        // HV CAN bus is configured for 500K
        bit_timing: BIT_TIMING,
    };

    let can1_tx = gpiod.pd1.into_alternate_af9();
    let can1_rx = gpiod.pd0.into_alternate_af9();

    let fc_can = Can::can1(p.CAN1, (can1_tx, can1_rx), &mut rcc.apb1, &HV_CAN_CONFIG)
        .expect("Failed to configure HV CAN (CAN1)");
    let can_filter: CanFilterConfig = CanFilterConfig::default();
    fc_can.configure_filter(&can_filter).ok();

    // RTC
    let rtc = Rtc::new(
        p.RTC,
        255,
        127,
        false,
        &mut rcc.apb1,
        &mut rcc.bdcr,
        &mut p.PWR,
    );

    return (fault_in, relay_1, relay_2, fc_can, serial, timer, rtc);
}

#[cfg(feature = "nucleof446re")]
pub fn init_devices() -> (
    FaultLinePin,
    RelayOnePin,
    RelayTwoPin,
    FCCAN,
    hal::serial::Serial<
        hal::stm32::USART2,
        (
            hal::gpio::gpioa::PA2<hal::gpio::Alternate<AF7>>,
            hal::gpio::gpioa::PA3<Alternate<AF7>>,
        ),
    >,
    hal::timer::Timer<hal::stm32::TIM2>,
    hal::rtc::Rtc,
) {
    // Hardware to initialize:
    // Fault Input
    // Latch Output
    // CAN Tx, Rx
    // Serial port
    // TIM2
    // RTC
    let mut p = pac::Peripherals::take().unwrap();
    let mut syscfg = p.SYSCFG;
    let mut exti = p.EXTI;

    // GPIO B for Fault and Latch I/O (PB3 for Fault (Read), and PB5 for Latch (Push-Pull High
    // output)). Also CAN Bus 1.
    let gpiob = p.GPIOB.split();
    let mut fault_in = gpiob.pb3.into_floating_input();
    fault_in.make_interrupt_source(&mut syscfg);

    let relay_1 = gpiob.pb5.into_push_pull_output();

    let mut relay_2 = gpiob.pb6.into_push_pull_output();
    relay_2.set_high().ok();

    // Set trigger and enable interrupt
    fault_in.trigger_on_edge(&mut exti, Edge::RISING_FALLING);
    fault_in.enable_interrupt(&mut exti);

    // Configure clocks
    let mut rcc = p.RCC.constrain();
    #[cfg(feature = "nucleof446re")]
    let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(180.mhz()).freeze();

    let gpioa = p.GPIOA.split();
    let tx_pin = gpioa.pa2.into_alternate_af7();
    let rx_pin = gpioa.pa3.into_alternate_af7();
    let serial = Serial::usart2(
        p.USART2,
        (tx_pin, rx_pin),
        Config::default().baudrate(230400.bps()),
        clocks,
    )
    .unwrap();

    // Timer
    let mut timer = Timer::tim2(p.TIM2, 1.khz(), clocks);
    timer.listen(Event::TimeOut);

    // -- CAN BUS --
    // Set up CAN bit timing.
    // Bit Timing for 180MHz System Clock
    // (45MHz APB1)
    // CAN_BTR: 0x001e0004
    #[cfg(feature = "nucleof446re")]
    const BIT_TIMING: CanBitTiming = CanBitTiming {
        prescaler: 4, // Prescaler: 5
        sjw: 0,       // CAN_SJW_1TQ
        bs1: 14,      // CAN_BS1_15TQ
        bs2: 1,       // CAN_BS2_2TQ
    };

    pub const HV_CAN_CONFIG: CanConfig = CanConfig {
        loopback_mode: false,
        silent_mode: false,
        ttcm: false,
        abom: true,
        awum: false,
        nart: false,
        rflm: false,
        txfp: false,
        // TODO - update CAN impl to calculate these
        // HV CAN bus is configured for 500K
        bit_timing: BIT_TIMING,
    };

    #[cfg(feature = "nucleof446re")]
    let can1_tx = gpiob.pb9.into_alternate_af9();
    #[cfg(feature = "nucleof446re")]
    let can1_rx = gpiob.pb8.into_alternate_af9();

    let fc_can = Can::can1(p.CAN1, (can1_tx, can1_rx), &mut rcc.apb1, &HV_CAN_CONFIG)
        .expect("Failed to configure HV CAN (CAN1)");
    let can_filter: CanFilterConfig = CanFilterConfig::default();
    fc_can.configure_filter(&can_filter).ok();

    // RTC
    let rtc = Rtc::new(
        p.RTC,
        255,
        127,
        false,
        &mut rcc.apb1,
        &mut rcc.bdcr,
        &mut p.PWR,
    );

    return (fault_in, relay_1, relay_2, fc_can, serial, timer, rtc);
}
