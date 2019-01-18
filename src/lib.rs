#define SPIFLASH_WRITEENABLE      0x06        // write enable
#define SPIFLASH_WRITEDISABLE     0x04        // write disable

#define SPIFLASH_BLOCKERASE_4K    0x20        // erase one 4K block of flash memory
#define SPIFLASH_BLOCKERASE_32K   0x52        // erase one 32K block of flash memory
#define SPIFLASH_BLOCKERASE_64K   0xD8        // erase one 64K block of flash memory
#define SPIFLASH_CHIPERASE        0x60        // chip erase (may take several seconds depending on size)
                                              // but no actual need to wait for completion (instead need to check the status register BUSY bit)
#define SPIFLASH_STATUSREAD       0x05        // read status register
#define SPIFLASH_STATUSWRITE      0x01        // write status register
#define SPIFLASH_ARRAYREAD        0x0B        // read array (fast, need to add 1 dummy byte after 3 address bytes)
#define SPIFLASH_ARRAYREADLOWFREQ 0x03        // read array (low frequency)

#define SPIFLASH_SLEEP            0xB9        // deep power down
#define SPIFLASH_WAKE             0xAB        // deep power wake up
#define SPIFLASH_BYTEPAGEPROGRAM  0x02        // write (1 to 256bytes)
#define SPIFLASH_IDREAD           0x9F        // read JEDEC manufacturer and device ID (2 bytes, specific bytes for each manufacturer and device)
                                              // Example for Atmel-Adesto 4Mbit AT25DF041A: 0x1F44 (page 27: http://www.adestotech.com/sites/default/files/datasheets/doc3668.pdf)
                                              // Example for Winbond 4Mbit W25X40CL: 0xEF30 (page 14: http://www.winbond.com/NR/rdonlyres/6E25084C-0BFE-4B25-903D-AE10221A0929/0/W25X40CL.pdf)
#define SPIFLASH_MACREAD          0x4B        // read unique ID number (MAC)

uint8_t SPIFlash::UNIQUEID[8];

/// IMPORTANT: NAND FLASH memory requires erase before write, because
///            it can only transition from 1s to 0s and only the erase command can reset all 0s to 1s
/// See http://en.wikipedia.org/wiki/Flash_memory
/// The smallest range that can be erased is a sector (4K, 32K, 64K); there is also a chip erase command

/// Constructor. JedecID is optional but recommended, since this will ensure that the device is present and has a valid response
/// get this from the datasheet of your flash chip
/// Example for Atmel-Adesto 4Mbit AT25DF041A: 0x1F44 (page 27: http://www.adestotech.com/sites/default/files/datasheets/doc3668.pdf)
/// Example for Winbond 4Mbit W25X40CL: 0xEF30 (page 14: http://www.winbond.com/NR/rdonlyres/6E25084C-0BFE-4B25-903D-AE10221A0929/0/W25X40CL.pdf)

struct SPIFlash {
    spi: SPI,
    cs: Pin,
    jedec_id: u16,
}

impl SPIFlash {
    pub fn new(spi: SPI, cs: Pin, jedec_id: u16) -> Self {
        Self {
            spi,
            cs,
            jedec_id
        }
    }

    pub fn unlock() {
        write_command(SPIFLASH_STATUSWRITE);
        SPI.transfer(0);
    }

    pub fn write_command(command: u8) {
        command(SPIFLASH_WRITEENABLE);
    }

    pub fn command(command: u8) {
        //wait for any write/erase to complete
        //  a time limit cannot really be added here without it being a very large safe limit
        //  that is because some chips can take several seconds to carry out a chip erase or other similar multi block or entire-chip operations
        //  a recommended alternative to such situations where chip can be or not be present is to add a 10k or similar weak pulldown on the
        //  open drain MISO input which can read noise/static and hence return a non 0 status byte, causing the while() to hang when a flash chip is not present
        if cmd != SPIFLASH_WAKE {
            while busy();
        }
        SPI.transfer(command);
    }

    pub fn read_device_id() {
        self.command(SPIFLASH_IDREAD);
        u16 jedec_id = SPI.transfer(0) << 8 | SPI.transfer(0);
        self.jedec_id = jedec_id;
    }

    pub fn read_unique_id() {
        self.command(SPIFLASH_MACREAD);
        SPI.transfer(0);
        SPI.transfer(0);
        SPI.transfer(0);
        SPI.transfer(0);
        for i in 0..8 {
            UNIQUEID[i] = SPI.transfer(0);
        }
    }

    pub fn read_byte(address: u32) -> u8 {
        self.command(SPIFLASH_ARRAYREADLOWFREQ);
        SPI.transfer(addr >> 16);
        SPI.transfer(addr >> 8);
        SPI.transfer(addr);
        u8 result = SPI.transfer(0);
    }

    pub fn read_bytes(address: u32, buffer: &mut [u8]) -> u8 {
        self.command(SPIFLASH_ARRAYREAD);
        SPI.transfer(addr >> 16);
        SPI.transfer(addr >> 8);
        SPI.transfer(addr);
        SPI.transfer(0); //"dont care"
        for i in 0..buffer.len() {
            buffer[i] = SPI.transfer(0);
        }
    }

    pub fn write_byte(address: u32, byte: u8) -> u8 {
        self.command_write(SPIFLASH_BYTEPAGEPROGRAM);
        SPI.transfer(addr >> 16);
        SPI.transfer(addr >> 8);
        SPI.transfer(addr);
        SPI.transfer(byte);
    }

    fn busy() -> bool {
        self.read_status() & 1 > 0
    }

    fn read_status() -> u8 {
        SPI.transfer(SPIFLASH_STATUSREAD);
        uint8_t status = SPI.transfer(0);
    }

    /// erase entire flash memory array
    /// may take several seconds depending on size, but is non blocking
    /// so you may wait for this to complete using busy() or continue doing
    /// other things and later check if the chip is done with busy()
    /// note that any command will first wait for chip to become available using busy()
    /// so no need to do that twice
    pub fn chip_erase() {
        self.command_write(SPIFLASH_CHIPERASE);
    }

    pub fn erase_4k_block(uint32_t addr) {
        self.command(SPIFLASH_BLOCKERASE_4K, true); // Block Erase
        SPI.transfer(addr >> 16);
        SPI.transfer(addr >> 8);
        SPI.transfer(addr);
    }

    pub fn erase_32k_block(uint32_t addr) {
        self.command(SPIFLASH_BLOCKERASE_32K, true); // Block Erase
        SPI.transfer(addr >> 16);
        SPI.transfer(addr >> 8);
        SPI.transfer(addr);
    }

    pub fn erase_64k_block(uint32_t addr) {
        self.command(SPIFLASH_BLOCKERASE_64K, true); // Block Erase
        SPI.transfer(addr >> 16);
        SPI.transfer(addr >> 8);
        SPI.transfer(addr);
    }

    pub fn sleep() {
        self.command(SPIFLASH_SLEEP);
    }

    fn wakeup() {
        self.command(SPIFLASH_WAKE);
    }
}

// /// write multiple bytes to flash memory (up to 64K)
// /// WARNING: you can only write to previously erased memory locations (see datasheet)
// ///          use the block erase commands to first clear memory (write 0xFFs)
// /// This version handles both page alignment and data blocks larger than 256 bytes.
// ///
// void SPIFlash::writeBytes(uint32_t addr, const void* buf, uint16_t len) {
//   uint16_t n;
//   uint16_t maxBytes = 256-(addr%256);  // force the first set of bytes to stay within the first page
//   uint16_t offset = 0;
//   while (len>0)
//   {
//     n = (len<=maxBytes) ? len : maxBytes;
//     command(SPIFLASH_BYTEPAGEPROGRAM, true);  // Byte/Page Program
//     SPI.transfer(addr >> 16);
//     SPI.transfer(addr >> 8);
//     SPI.transfer(addr);
    
//     for (uint16_t i = 0; i < n; i++)
//       SPI.transfer(((uint8_t*) buf)[offset + i]);
//     unselect();
    
//     addr+=n;  // adjust the addresses and remaining bytes by what we've just transferred.
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
