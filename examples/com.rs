#![no_main]
#![no_std]

use cortex_m_rt::entry;
use nb::block;

#[allow(unused_imports)]
use panic_semihosting;

// use cortex_m_semihosting::hprintln;

use nrf52840_hal::{
    uarte::{
        Error
    },
    nrf52840_pac::{
        self as nrf52,
    },
};
use nrf52840_dk_bsp::{
    hal::{
        prelude::*,
        timer::Timer,
    },
    nRF52840DK,
};

#[entry]
fn main() -> ! {
    let mut nrf52 = nRF52840DK::take().unwrap();

    let mut timer = nrf52.TIMER0.constrain();

    let mut uarte = nrf52.com;

    // Alternately flash the red and blue leds
    loop {
        // nrf52.leds.led_2.enable();
        // delay(&mut timer, 1_000_000); // 250ms
        // nrf52.leds.led_2.disable();
        // delay(&mut timer, 1_000_000); // 1s
        let e = uarte.write(&[65, 65, 65]);
        match e {
            Err(Error::TxBufferTooLong) => nrf52.leds.led_1.enable(),
            Err(Error::RxBufferTooLong) => nrf52.leds.led_2.enable(),
            Err(Error::Transmit) => nrf52.leds.led_3.enable(),
            Err(Error::Receive) => nrf52.leds.led_4.enable(),
            Ok(_) => {
                nrf52.leds.led_2.enable();
                nrf52.leds.led_1.enable();
                delay(&mut timer, 1_000_000); // 250ms
                nrf52.leds.led_1.disable();
                nrf52.leds.led_2.disable();
                delay(&mut timer, 1_000_000); // 1s
            }
        }
        // hprintln!("Hello, world!").unwrap();
    }
}

fn delay<T>(timer: &mut Timer<T>, cycles: u32)
where
    T: TimerExt,
{
    timer.start(cycles);
    let _ = block!(timer.wait());
}