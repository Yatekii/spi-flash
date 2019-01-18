#![no_std]

const SPIFLASH_WRITEENABLE: u8 = 0x06;        // write enable
const SPIFLASH_WRITEDISABLE: u8 = 0x04;        // write disable

const SPIFLASH_BLOCKERASE_4K: u8 = 0x20;        // erase one 4K block of flash memory
const SPIFLASH_BLOCKERASE_32K: u8 = 0x52;        // erase one 32K block of flash memory
const SPIFLASH_BLOCKERASE_64K: u8 = 0xD8;        // erase one 64K block of flash memory
const SPIFLASH_CHIPERASE: u8 =  0x60;        // chip erase (may take several seconds depending on size)
                                              // but no actual need to wait for completion (instead need to check the status register BUSY bit)
const SPIFLASH_STATUSREAD: u8 = 0x05;        // read status register
const SPIFLASH_STATUSWRITE: u8 = 0x01;        // write status register
const SPIFLASH_ARRAYREAD: u8 = 0x0B;        // read array (fast, need to add 1 dummy byte after 3 address bytes)
const SPIFLASH_ARRAYREADLOWFREQ: u8 = 0x03;        // read array (low frequency)

const SPIFLASH_SLEEP: u8 = 0xB9;        // deep power down
const SPIFLASH_WAKE: u8 = 0xAB;        // deep power wake up
const SPIFLASH_BYTEPAGEPROGRAM: u8 = 0x02;        // write (1 to 256bytes)
const SPIFLASH_IDREAD: u8 = 0x9F;        // read JEDEC manufacturer and device ID (2 bytes, specific bytes for each manufacturer and device)
                                              // Example for Atmel-Adesto 4Mbit AT25DF041A: 0x1F44 (page 27: http://www.adestotech.com/sites/default/files/datasheets/doc3668.pdf)
                                              // Example for Winbond 4Mbit W25X40CL: 0xEF30 (page 14: http://www.winbond.com/NR/rdonlyres/6E25084C-0BFE-4B25-903D-AE10221A0929/0/W25X40CL.pdf)
const SPIFLASH_MACREAD: u8 = 0x4B;        // read unique ID number (MAC)

/// IMPORTANT: NAND FLASH memory requires erase before write, because
///            it can only transition from 1s to 0s and only the erase command can reset all 0s to 1s
/// See http://en.wikipedia.org/wiki/Flash_memory
/// The smallest range that can be erased is a sector (4K, 32K, 64K); there is also a chip erase command

/// Constructor. JedecID is optional but recommended, since this will ensure that the device is present and has a valid response
/// get this from the datasheet of your flash chip
/// Example for Atmel-Adesto 4Mbit AT25DF041A: 0x1F44 (page 27: http://www.adestotech.com/sites/default/files/datasheets/doc3668.pdf)
/// Example for Winbond 4Mbit W25X40CL: 0xEF30 (page 14: http://www.winbond.com/NR/rdonlyres/6E25084C-0BFE-4B25-903D-AE10221A0929/0/W25X40CL.pdf)

use nb::block;
use nb;
use embedded_hal::{
    digital::OutputPin,
    spi::FullDuplex,
};

pub struct SPIFlash<CS, SPI>
where
    CS: OutputPin,
    SPI: FullDuplex<u8>,
{
    spi: SPI,
    cs: CS,
}

impl<CS, SPI> SPIFlash<CS, SPI>
where
    CS: OutputPin,
    SPI: FullDuplex<u8>,
{
    pub fn new(spi: SPI, cs: CS) -> Self {
        Self {
            spi,
            cs
        }
    }

    pub fn unlock(&mut self) {
        self.send(SPIFLASH_STATUSWRITE);
        self.send(0);
    }

    pub fn read_byte(&mut self, address: u32) -> u8 {
        self.send(SPIFLASH_ARRAYREADLOWFREQ);
        self.send((address >> 16) as u8);
        self.send((address >> 8) as u8);
        self.send((address) as u8);
        self.send(0);
        // TODO: Fix! UNSAFE!
        if let Ok(byte) = block!(self.spi.read()) {
            byte
        } else {
            0
        }
    }

    // pub fn read_bytes(address: u32, buffer: &mut [u8]) -> u8 {
    //     self.command(SPIFLASH_ARRAYREAD);
    //     SPI.transfer(address >> 16);
    //     SPI.transfer(address >> 8);
    //     SPI.transfer(address);
    //     SPI.transfer(0); //"dont care"
    //     for i in 0..buffer.len() {
    //         buffer[i] = SPI.transfer(0);
    //     }
    // }

    pub fn write_byte(&mut self, address: u32, byte: u8) {
        self.send(SPIFLASH_BYTEPAGEPROGRAM);
        self.send((address >> 16) as u8);
        self.send((address >> 8) as u8);
        self.send((address) as u8);
        self.send(byte);
    }

    /// Checks whether the SPI flash is busy.
    /// Returns `true` if it is still busy.
    fn busy(&mut self) -> bool {
        self.read_status() & 1 > 0
    }

    /// Waits for the SPI flash to complete it's current action.
    /// Supports the async API of the `nb` crate.
    fn wait(&mut self) -> nb::Result<(), ()> {
        if self.busy() {
            Err(nb::Error::WouldBlock)
        } else {
            Ok(())
        }
    }

    fn read_status(&mut self) -> u8 {
        self.send(SPIFLASH_STATUSREAD);
        // TODO: Fix! UNSAFE!
        self.read()
    }

    /// erase entire flash memory array
    /// may take several seconds depending on size, but is non blocking
    /// so you may wait for this to complete using busy() or continue doing
    /// other things and later check if the chip is done with busy()
    /// note that any command will first wait for chip to become available using busy()
    /// so no need to do that twice
    pub fn chip_erase(&mut self) {
        self.send(SPIFLASH_CHIPERASE);
        let _ = block!(self.wait());
    }

    /// Erase a 4k block of the memory.
    pub fn erase_4k_block(&mut self, address: u32) {
        // TODO: write command;
        self.send(SPIFLASH_BLOCKERASE_4K);
        self.send((address >> 16) as u8);
        self.send((address >> 8) as u8);
        self.send((address) as u8);
    }

    // pub fn erase_32k_block(uint32_t address) {
    //     self.command(SPIFLASH_BLOCKERASE_32K, true); // Block Erase
    //     SPI.transfer(address >> 16);
    //     SPI.transfer(address >> 8);
    //     SPI.transfer(address);
    // }

    // pub fn erase_64k_block(uint32_t address) {
    //     self.command(SPIFLASH_BLOCKERASE_64K, true); // Block Erase
    //     SPI.transfer(address >> 16);
    //     SPI.transfer(address >> 8);
    //     SPI.transfer(address);
    // }

    pub fn sleep(&mut self) {
        self.send(SPIFLASH_SLEEP);
    }

    pub fn wakeup(&mut self) {
        self.send(SPIFLASH_WAKE);
    }

    fn send(&mut self, byte: u8) {
        let _ = block!(self.spi.send(byte));
    }

    fn read(&mut self) -> u8 {
        if let Ok(b) = block!(self.spi.read()) {
            b
        } else {
            0
        }
    }
}

// /// write multiple bytes to flash memory (up to 64K)
// /// WARNING: you can only write to previously erased memory locations (see datasheet)
// ///          use the block erase commands to first clear memory (write 0xFFs)
// /// This version handles both page alignment and data blocks larger than 256 bytes.
// ///
// void SPIFlash::writeBytes(uint32_t address, const void* buf, uint16_t len) {
//   uint16_t n;
//   uint16_t maxBytes = 256-(address%256);  // force the first set of bytes to stay within the first page
//   uint16_t offset = 0;
//   while (len>0)
//   {
//     n = (len<=maxBytes) ? len : maxBytes;
//     command(SPIFLASH_BYTEPAGEPROGRAM, true);  // Byte/Page Program
//     SPI.transfer(address >> 16);
//     SPI.transfer(address >> 8);
//     SPI.transfer(address);
    
//     for (uint16_t i = 0; i < n; i++)
//       SPI.transfer(((uint8_t*) buf)[offset + i]);
//     unselect();
    
//     address+=n;  // adjust the addresses and remaining bytes by what we've just transferred.
//     offset +=n;
//     len -= n;
//     maxBytes = 256;   // now we can do up to 256 bytes per loop
//   }
// }

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
