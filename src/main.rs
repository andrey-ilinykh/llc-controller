#![no_std]
#![no_main]

use panic_halt as _; // panic handler

use cortex_m;
use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{pac, prelude::*, timer::Channel1, timer::Timer};

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

    let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

    let mut delay = cp.SYST.delay(&clocks);

    let gpioc = dp.GPIOC.split();
    let gpioa = dp.GPIOA.split();
    let mut led = gpioc.pc13.into_push_pull_output();
    let channels = Channel1::new(gpioa.pa8).with_complementary(gpioa.pa7);

    let mut pwm = dp.TIM1.pwm_hz(channels, 70.kHz(), &clocks);

    let mut max_duty: u16 = pwm.get_max_duty();

    pwm.set_polarity(Channel::C1, Polarity::ActiveHigh);
    pwm.set_complementary_polarity(Channel::C1, Polarity::ActiveHigh);

    pwm.set_duty(Channel::C1, max_duty / 2);

    pwm.set_dead_time(5);

    pwm.enable(Channel::C1);
    pwm.enable_complementary(Channel::C1);

    loop {
        // Status LED blink
        led.set_high();
        delay.delay_ms(50u32);
        led.set_low();
        delay.delay_ms(50u32);

        delay.delay_ms(100u32);
    }
}
