#![no_std]
#![no_main]

use nb::block;
use panic_halt as _; // panic handler

use cortex_m::{self};
use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{
    self as hal,
     timer::{Tim1NoRemap, Timer},
};

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


    let pa8 = gpioa.pa8.into_alternate_push_pull(&mut gpioa.crh); // TIM1_CH1
    let _pb13 = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh); // TIM1_CH1N

    // Set up TIM1 PWM
    let pwm = dp.TIM1.pwm_hz::<Tim1NoRemap, _, _>(
        pa8,
        &mut afio.mapr,
        200.kHz(),
        &clocks,
       
    );

    let mut pwm_ch1 = pwm.split();
    
    let max = pwm_ch1.get_max_duty();
    pwm_ch1.set_duty(max / 2); // 50% duty
    pwm_ch1.enable();

    // PAC access to TIM1
    let tim1 = unsafe { &*pac::TIM1::ptr() };

    // 1. Enable CH1N (complementary)
    tim1.ccer.modify(|_, w| {
        w.cc1e().set_bit();   // Enable CH1
        w.cc1ne().set_bit();  // Enable CH1N
        w.cc1p().clear_bit();  // CH1 active high
        w.cc1np().clear_bit(); // CH1N active high
        w
    });

    // 2. Set dead time and enable main output
    tim1.bdtr.modify(|_, w| unsafe {
        w.dtg().bits(8)        // ~100 ns at 72 MHz → 10 × 13.8 ns = ~138 ns
         .ossi().set_bit()      // Off-state output enabled
         .moe().set_bit()       // Main Output Enable
    });

    let mut dma = dp.DMA1.split();
    static mut BURST_BUF: [u16; 4] = [0b0101, 0b0, 0b0, 0b0]; // CH1+CH1N on, then off
    
    

    unsafe {
        dma.3.ch().cr.modify(|_, w| {
            w.en().clear_bit() // Disable channel
          
        });
       
       dma.3.ch().par.write(|w| w.bits(&tim1.ccer as *const _ as u32));
       dma.3.ch().mar
        .write(|w| w.bits(BURST_BUF.as_ptr() as u32));

    

    


       dma.3.ch().ndtr
        .write(|w| w.ndt().bits(4));

        dma.3.ch().cr.modify(|_, w| {
            w.mem2mem().clear_bit() // Memory to peripheral
             .pl().very_high()              // Medium priority
             .msize().bits16()           // Memory: 16-bit
             .psize().bits16()  
             .minc().set_bit() // Memory increment mode
             .pinc().clear_bit() // Peripheral not incremented
             .circ().set_bit() // Circular mode
             .dir().set_bit() // Memory to peripheral direction
             
        });
        
    }

    dma.3.ch().cr.modify(|_, w| {
            w.en().set_bit() // EnableDisable channel
          //   .tcie().set_bit() // Transfer complete interrupt enabled
        });
    
    
    let arr = tim1.arr.read().bits();
    tim1.ccr2().write(|w| unsafe { w.bits(arr -5) }); // 5 ticks before overflow
    tim1.ccmr1_output().modify(|_, w| w.oc2pe().clear_bit()); // no preload


    

    tim1.ccer.modify(|_, w| w.cc2e().clear_bit()); // No output
    tim1.dier.modify(|_, w| w.cc2de().set_bit()); // Enable DMA on CCR2
   


    // Enable counter
    tim1.cr1.modify(|_, w| w.cen().set_bit());
   

    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(10.Hz()).unwrap();
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    loop {
      for _ in 0..10 {
        block!(timer.wait()).unwrap();
        }
 //     tim1.ccer.modify(|_, w| w.cc1e().set_bit()); // Enable CH1
        let ccr1 = tim1.ccr1().read().bits();
        

        rprintln!("ccr1: {:#06X}, arr: {:#06X}", max, arr);
       // rprintln!("LED ON");
        led.set_high();
        block!(timer.wait()).unwrap();
  //      tim1.ccer.modify(|_, w| w.cc1e().clear_bit()); // Disable CH1
        led.set_low();
    }
}
