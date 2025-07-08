#![no_std]
#![no_main]

use panic_halt as _; // panic handler

use cortex_m;
use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{
    self as hal,
    dma::{config::DmaConfig, traits::DMASet, MemoryToPeripheral, StreamX, StreamsTuple, Transfer},
    pac::{DMA2, SPI2, TIM1},
    spi::Spi,
};

use hal::{pac, prelude::*, timer::Polarity};
const DUTY_MAX: u16 = 2400;
struct TIM1CCR1 {}

impl TIM1CCR1 {
    pub fn new() -> Self {
        TIM1CCR1 {}
    }
}
unsafe impl hal::dma::traits::PeriAddress for TIM1CCR1 {
    fn address(&self) -> u32 {
        TIM1::ptr() as u32 + 0x34 // CCR1 offset
    }
    type MemSize = u16; // Memory size is u16 for CCR1
}

unsafe impl DMASet<StreamX<DMA2, 5>, 5, MemoryToPeripheral> for TIM1CCR1 {}

static mut DUTY_PATTERN: [u16; 2] = [0, DUTY_MAX]; // off/on modulation
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
    let (mut pwm_mngr, (pwm_c1, ..)) = dp.TIM1.pwm_hz(200.kHz(), &mut clocks);

    let mut pwm_c1 = pwm_c1.with(gpioa.pa8).with_complementary(gpioa.pa7);

    let max_duty: u16 = pwm_c1.get_max_duty();
    unsafe {
        DUTY_PATTERN[1] = max_duty / 2; // off
    }

    pwm_c1.set_polarity(Polarity::ActiveHigh);
    pwm_c1.set_complementary_polarity(Polarity::ActiveHigh);

    pwm_c1.set_duty(max_duty / 2);

    pwm_mngr.set_dead_time(5);

    pwm_c1.enable();
    pwm_c1.enable_complementary();

    let mut led = gpioc.pc13.into_push_pull_output();
    let mut delay = dp.TIM5.delay_us(&clocks);

    // Enable DMA trigger on TIM1 update
    let tim1 = unsafe { &*pac::TIM1::ptr() };
    tim1.dier().modify(|_, w| w.ude().set_bit()); // Update DMA request

    //tim1.dier().modify(|_, w| w.ude().set_bit()); // Update DMA request

    // Set up DMA to write to CCR1
    let streams = StreamsTuple::new(dp.DMA2);
    let mut dma_stream = streams.5; // Stream5 for TIM1_UP (ch6)
    dma_stream.set_circular_mode(true);

    let dma_cfg = DmaConfig::default()
        .memory_increment(true)
        .peripheral_increment(false);

    let peripheral = TIM1CCR1::new();

    // SAFETY: DUTY_PATTERN is only used here and not aliased elsewhere
    let mut transfer = Transfer::init_memory_to_peripheral(
        dma_stream,
        peripheral,
        unsafe { &mut DUTY_PATTERN }, // source must be &'static mut [u16]
        // destination: HAL abstraction for CCR1
        None, // no double buffer
        dma_cfg,
    );
    let mut transfer = transfer.start(|_s| {});

    loop {
        // Status LED blink
        led.set_high();
        delay.delay_ms(50u32);
        led.set_low();
        delay.delay_ms(50u32);

        delay.delay_ms(100u32);
    }
}
