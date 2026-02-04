use crate::ata::AtaDevice;
use crate::controllers::{is_bank_empty, FlashBank, FlashIdResult};

const BANK_SIZE: usize = 8;
const MAX_BANKS: usize = 16;

// Yeestor/SiliconGo magic LBA patterns: (lba_mid, lba_low)
// Uses READ DMA (0xC8) with Device=0x40 instead of READ SECTORS with Device=0xE0
const MAGIC_LBAS: &[(u8, u8)] = &[
    (0x55, 0x00), // R1
    (0x00, 0x55), // R2
    (0xAA, 0x00), // R3
    (0x00, 0xAA), // R4
    (0x55, 0xAA), // R5
];

fn read_magic_dma(dev: &AtaDevice, lba_mid: u8, lba_low: u8) -> Result<[u8; 512], String> {
    let mut buf = [0u8; 512];
    dev.ata_dma_read(0xC8, 0x00, 1, lba_low, lba_mid, 0x00, 0x40, &mut buf)?;
    Ok(buf)
}

pub fn read_flash_id(dev: &AtaDevice) -> Result<FlashIdResult, String> {
    let jedec_manufacturers = [0x2C, 0x89, 0xAD, 0x45, 0xEC, 0x98, 0xC8, 0x9B, 0x01];

    for &(mid, low) in MAGIC_LBAS {
        let buf = match read_magic_dma(dev, mid, low) {
            Ok(b) => b,
            Err(_) => continue,
        };

        // Verify first byte is a known JEDEC manufacturer ID
        if !jedec_manufacturers.contains(&buf[0]) {
            continue;
        }

        let mut banks = Vec::new();
        for bank_num in 0..MAX_BANKS {
            let offset = bank_num * BANK_SIZE;
            if offset + BANK_SIZE > buf.len() {
                break;
            }
            let bank_data = &buf[offset..offset + BANK_SIZE];
            if !is_bank_empty(bank_data) {
                let mut flash_id = [0u8; 8];
                flash_id.copy_from_slice(bank_data);
                banks.push(FlashBank {
                    bank_num: bank_num as u32,
                    flash_id,
                });
            } else {
                break;
            }
        }

        if !banks.is_empty() {
            return Ok(FlashIdResult {
                controller_name: format!(
                    "Yeestor/SiliconGo (DMA 0x{:02X}{:02X})",
                    mid, low
                ),
                banks,
            });
        }
    }

    Err("no flash ID data found in Yeestor/SiliconGo vendor responses".to_string())
}
