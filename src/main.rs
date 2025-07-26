#![no_std]
#![no_main]

use nb::block;
use panic_halt as _; // panic handler

use cortex_m::{self};
use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::dma::dma1::Channels;
use stm32f1xx_hal::timer::pwm::Pins;
use stm32f1xx_hal::{
    self as hal,
    afio::MAPR,
    gpio::{self, Alternate},
    rcc::Clocks,
    time::Hertz,
    timer::{CPin, Ch, PwmChannel, PwmHz, Tim1NoRemap, Timer},
};
static mut BURST_BUF: [u16; 4] = [0b0101, 0b0, 0b0, 0b0]; // CH1+CH1N on, then off
struct PwmController<'a, const C: u8> {
    tim1: &'a pac::tim1::RegisterBlock,
    dma: Channels,
    pwm_ch1: PwmChannel<pac::TIM1, C>,
}

impl<'a, const C: u8> PwmController<'a, C> {
    fn new<PINS>(
        tim: pac::TIM1,
        pin: PINS,
        mapr: &mut MAPR,
        freq: Hertz,
        clocks: &Clocks,
        dma: Channels,
    ) -> Self
    where
        PINS: Pins<Tim1NoRemap, Ch<C>, Channels = PwmChannel<pac::TIM1, C>>
            + gpio::PinExt<Mode = Alternate>,
    {
        let tim1 = unsafe { &*pac::TIM1::ptr() };
        let pwm: PwmHz<pac::TIM1, Tim1NoRemap, Ch<C>, PINS> =
            tim.pwm_hz::<Tim1NoRemap, Ch<C>, PINS>(pin, mapr, freq, clocks);

        let mut pwm_ch1: PwmChannel<pac::TIM1, C> = pwm.split();
        // pwm_ch1.enable();
        PwmController {
            tim1,
            dma,
            pwm_ch1,
            //_p: core::marker::PhantomData,
        }
    }

    fn enable(&mut self) {
        self.tim1.ccer.modify(|_, w| {
            w.cc1e().set_bit(); // Enable CH1
            w.cc1ne().set_bit(); // Enable CH1N
            w
        });
    }

    fn disable(&mut self) {
        self.tim1.ccer.modify(|_, w| {
            w.cc1e().clear_bit(); // Disable CH1
            w.cc1ne().clear_bit(); // Disable CH1N
            w
        });
    }

    fn init(&mut self) {
        self.pwm_ch1.set_duty(self.pwm_ch1.get_max_duty() / 2); // 50% duty

        self.tim1.bdtr.modify(|_, w| unsafe {
            w.dtg()
                .bits(8) // ~100 ns at 72 MHz → 10 × 13.8 ns = ~138 ns
                .ossi()
                .set_bit() // Off-state output enabled
                .moe()
                .set_bit() // Main Output Enable
        });

        self.tim1.ccer.modify(|_, w| {
            //  w.cc1e().set_bit(); // Enable CH1
            //  w.cc1ne().set_bit(); // Enable CH1N
            w.cc1p().clear_bit(); // CH1 active high
            w.cc1np().clear_bit(); // CH1N active high
            w
        });

        self.dma.3.ch().cr.modify(|_, w| {
            w.en().clear_bit() // Disable channel
        });

        unsafe {
            self.dma
                .3
                .ch()
                .par
                .write(|w| w.bits(&self.tim1.ccer as *const _ as u32));

            self.dma
                .3
                .ch()
                .mar
                .write(|w| w.bits(BURST_BUF.as_ptr() as u32));
        }
        self.dma.3.ch().ndtr.write(|w| w.ndt().bits(4));

        self.dma.3.ch().cr.modify(|_, w| {
            w.mem2mem()
                .clear_bit() // Memory to peripheral
                .pl()
                .very_high() // Medium priority
                .msize()
                .bits16() // Memory: 16-bit
                .psize()
                .bits16()
                .minc()
                .set_bit() // Memory increment mode
                .pinc()
                .clear_bit() // Peripheral not incremented
                .circ()
                .set_bit() // Circular mode
                .dir()
                .set_bit() // Memory to peripheral direction
        });

        let arr = self.tim1.arr.read().bits();
        self.tim1.ccr2().write(|w| unsafe { w.bits(arr - 5) }); // 5 ticks before overflow
        self.tim1
            .ccmr1_output()
            .modify(|_, w| w.oc2pe().clear_bit()); // no preload

        self.tim1.ccer.modify(|_, w| w.cc2e().clear_bit()); // No output
        self.tim1.dier.modify(|_, w| w.cc2de().set_bit()); // Enable DMA on CCR2

        // Enable counter
        self.tim1.cr1.modify(|_, w| w.cen().set_bit());
    }
}

use hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    rtt_init_print!(); // Initialize RTT for printing
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Set up clocks
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(72.MHz()).freeze(&mut flash.acr);

    // Set up GPIO
    let mut afio = dp.AFIO.constrain();
    let mut gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();
    let mut gpioc = dp.GPIOC.split();

    // let pa9 = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh); // TIM1_CH1

    // let foo = Foo::new(dp.TIM1, pa9, &mut afio.mapr, 200.kHz(), &clocks);
    // let x: PwmHz<pac::TIM1, Tim1NoRemap, Ch<1>, gpio::Pin<'A', 9, Alternate>> = dp
    //     .TIM1
    //     .pwm_hz::<Tim1NoRemap, Ch<1>, _>(pa9, &mut afio.mapr, 200.kHz(), &clocks);

    // let xx: PwmChannel<pac::TIM1, 1> = x.split();

    let pa8 = gpioa.pa8.into_alternate_push_pull(&mut gpioa.crh); // TIM1_CH1
    let _pb13 = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh); // TIM1_CH1N
                                                                     // let foo = Foo1::new(dp.TIM1, pa8, &mut afio.mapr, 200.kHz(), &clocks);

    let dma: stm32f1xx_hal::dma::dma1::Channels = dp.DMA1.split();
    let mut pwc = PwmController::new(dp.TIM1, pa8, &mut afio.mapr, 200.kHz(), &clocks, dma);

    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(10.Hz()).unwrap();
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    pwc.enable();
    loop {
        for _ in 0..10 {
            block!(timer.wait()).unwrap();
        }
        //     tim1.ccer.modify(|_, w| w.cc1e().set_bit()); // Enable CH1

        // rprintln!("LED ON");
        led.set_high();
        block!(timer.wait()).unwrap();
        //      tim1.ccer.modify(|_, w| w.cc1e().clear_bit()); // Disable CH1
        led.set_low();
    }
}
