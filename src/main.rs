#![no_std]
#![no_main]

use panic_halt as _; // panic handler

use cortex_m;
use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal as hal;

use hal::{pac, prelude::*, timer::Polarity};

#[entry]
fn main() -> ! {
    // Initialize RTT for printing debug messages
    rtt_init_print!();

    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let gpioa = dp.GPIOA.split();
    let gpioc = dp.GPIOC.split();
    let rcc = dp.RCC.constrain();
    //let clocks = rcc.cfgr.sysclk(25.MHz()).freeze();
    let mut clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(48.MHz()).freeze();
    let (mut pwm_mngr, (pwm_c1, ..)) = dp.TIM1.pwm_hz(20.kHz(), &mut clocks);

    let mut pwm_c1 = pwm_c1.with(gpioa.pa8).with_complementary(gpioa.pa7);

    let max_duty: u16 = pwm_c1.get_max_duty();

    pwm_c1.set_polarity(Polarity::ActiveHigh);
    pwm_c1.set_complementary_polarity(Polarity::ActiveHigh);

    pwm_c1.set_duty(max_duty / 2);

    pwm_mngr.set_dead_time(200);

    pwm_c1.enable();
    pwm_c1.enable_complementary();

    let mut led = gpioc.pc13.into_push_pull_output();
    let mut delay = dp.TIM5.delay_us(&clocks);

    loop {
        // Status LED blink
        led.set_high();
        delay.delay_ms(50u32);
        led.set_low();
        delay.delay_ms(50u32);

        delay.delay_ms(100u32);
    }
}
