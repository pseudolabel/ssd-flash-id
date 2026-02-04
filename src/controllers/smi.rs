use crate::controllers::{is_bank_empty, FlashBank, FlashIdResult};
use crate::nvme::NvmeDevice;

const VALID_MANUFACTURER_IDS: &[u8] = &[
    0x01, 0x04, 0x07, 0x20, 0x2C, 0x45, 0x4A, 0x51, 0x89, 0x92, 0x98, 0x9B, 0xAD, 0xB5, 0xC2,
    0xC8, 0xEC, 0xEF,
];

const BANK_START: usize = 0x30;
const BANK_END: usize = 0x1F0;
const BANK_SIZE: usize = 8;
const CTRL_NAME_OFFSET: usize = 0x1F0;

pub fn read_flash_id(dev: &NvmeDevice) -> Result<FlashIdResult, String> {
    let mut buf = [0u8; 2048];
    dev.admin_read(0xC2, 0, 0x200, 0, 0x40, 0x01, 0, 0, &mut buf)
        .map_err(|e| format!("SMI flash ID command failed: {}", e))?;

    let ctrl_name = extract_controller_name(&buf[CTRL_NAME_OFFSET..]);

    let mut banks = Vec::new();
    let mut offset = BANK_START;
    let mut bank_num = 0u32;
    while offset + BANK_SIZE <= BANK_END {
        let bank_data = &buf[offset..offset + BANK_SIZE];
        if !is_bank_empty(bank_data) && VALID_MANUFACTURER_IDS.contains(&bank_data[0]) {
            let mut flash_id = [0u8; 8];
            flash_id.copy_from_slice(bank_data);
            banks.push(FlashBank {
                bank_num,
                flash_id,
            });
        }
        offset += BANK_SIZE;
        bank_num += 1;
    }

    Ok(FlashIdResult {
        controller_name: ctrl_name,
        banks,
    })
}

fn extract_controller_name(data: &[u8]) -> String {
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    let s: String = data[..end]
        .iter()
        .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { ' ' })
        .collect();
    let trimmed = s.trim();
    if trimmed.is_empty() {
        "SMI".to_string()
    } else {
        trimmed.to_string()
    }
}
