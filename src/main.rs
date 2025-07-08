#![no_std]
#![no_main]

use panic_halt as _; // panic handler

use cortex_m;
use cortex_m_rt::entry;
use stm32f4xx_hal::{
    pac,
    prelude::*,
    timer::Timer,
};
use rtt_target::{rtt_init_print, rprintln};

#[entry]
fn main() -> ! {
    // Initialize RTT for printing debug messages
    rtt_init_print!();

    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and use HSE (25 MHz) as the clock source
    let clocks = rcc.cfgr.use_hse(25.MHz()).freeze();

    // Acquire the GPIO peripherals
    let gpioa = dp.GPIOA.split();
    let gpioc = dp.GPIOC.split();

    // Configure PC13 as output for status LED
    let mut led = gpioc.pc13.into_push_pull_output();

    

    // Configure PA8 as alternate function for TIM1_CH1


    // Create a delay abstraction based on SysTick
    let mut delay = cp.SYST.delay(&clocks);

    

    loop {
        // Status LED blink
        led.set_high();
        delay.delay_ms(50u32);
        led.set_low();
        delay.delay_ms(50u32);

        
        delay.delay_ms(100u32);
    }
}
