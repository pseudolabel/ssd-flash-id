use crate::ata::AtaDevice;
use crate::controllers::{is_bank_empty, FlashBank, FlashIdResult};

const BANK_SIZE: usize = 8;
const MAX_BANKS: usize = 16;

fn smart_write_log(dev: &AtaDevice, log_addr: u8, payload: &[u8; 512]) -> Result<(), String> {
    // SMART WRITE LOG: Command=0xB0, Features=0xD6, Count=1
    // LBA_Low=log_addr, LBA_Mid=0x4F, LBA_High=0xC2 (SMART signature)
    dev.ata_write(0xB0, 0xD6, 0x01, log_addr, 0x4F, 0xC2, 0x00, payload)
}

fn smart_read_log(dev: &AtaDevice, log_addr: u8) -> Result<[u8; 512], String> {
    let mut buf = [0u8; 512];
    // SMART READ LOG: Command=0xB0, Features=0xD5, Count=1
    // LBA_Low=log_addr, LBA_Mid=0x4F, LBA_High=0xC2 (SMART signature)
    dev.ata_read(0xB0, 0xD5, 0x01, log_addr, 0x4F, 0xC2, 0x00, &mut buf)?;
    Ok(buf)
}

pub fn read_flash_id(dev: &AtaDevice) -> Result<FlashIdResult, String> {
    // Send flash ID request to SMART vendor log 0xE0
    let mut payload = [0u8; 512];
    payload[0] = 0x01;
    payload[1] = 0x34;
    payload[2] = 0xC0;

    smart_write_log(dev, 0xE0, &payload)
        .map_err(|e| format!("SandForce SMART WRITE LOG failed: {}", e))?;

    // Read flash ID response from SMART vendor log 0xE1
    let response = smart_read_log(dev, 0xE1)
        .map_err(|e| format!("SandForce SMART READ LOG 0xE1 failed: {}", e))?;

    if response.iter().all(|&b| b == 0x00 || b == 0xFF) {
        return Err("SandForce flash ID response is empty".to_string());
    }

    let mut banks = Vec::new();
    for bank_num in 0..MAX_BANKS {
        let offset = bank_num * BANK_SIZE;
        if offset + BANK_SIZE > response.len() {
            break;
        }
        let bank_data = &response[offset..offset + BANK_SIZE];
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

    if banks.is_empty() {
        return Err("no flash ID data found in SandForce response".to_string());
    }

    Ok(FlashIdResult {
        controller_name: "SandForce".to_string(),
        banks,
    })
}
