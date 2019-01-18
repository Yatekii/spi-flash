#![no_main]
#![no_std]

use cortex_m_rt::entry;
use nb::block;

#[allow(unused_imports)]
use panic_semihosting;

use nrf52840_hal::{
    spim::{
        Spim,
    },
    gpio::{
        p0,
        Output,
        PushPull,
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

use spi_flash::{
    self,
    Transmitter
};

struct SPITransmitter {
    spi: Spim<nrf52::SPIM2>,
    cs: p0::P0_Pin<Output<PushPull>>,
}

impl SPITransmitter {
    pub fn new(spi: Spim<nrf52::SPIM2>, cs: p0::P0_Pin<Output<PushPull>>) -> SPITransmitter {
        SPITransmitter { spi, cs }
    }
}

impl Transmitter for SPITransmitter {
    fn send(&mut self, buffer: &[u8]) {
        self.spi.write(&mut self.cs, buffer);
    }

    fn read(&mut self, buffer: &mut [u8]) {
        self.spi.read(&mut self.cs, &[], buffer);
    }

    fn send_read(&mut self, buffer_tx: &[u8], buffer_rx: &mut [u8]) {
        self.spi.read(&mut self.cs, buffer_tx, buffer_rx);
    }
}

#[entry]
fn main() -> ! {
    let mut nrf52 = nRF52840DK::take().unwrap();

    let mut timer = nrf52.TIMER0.constrain();

    let mut flash = spi_flash::SPIFlash::new(SPITransmitter::new(nrf52.flash, nrf52.flash_cs));

    let status = flash.read_status();

    let kek = 3;

    // Alternately flash the red and blue leds
    loop {
        nrf52.leds.led_2.enable();
        delay(&mut timer, 1_000_000); // 250ms
        nrf52.leds.led_2.disable();
        delay(&mut timer, 1_000_000); // 1s
    }
}

fn delay<T>(timer: &mut Timer<T>, cycles: u32)
where
    T: TimerExt,
{
    timer.start(cycles);
    let _ = block!(timer.wait());
}