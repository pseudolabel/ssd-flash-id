use crate::controllers::{is_bank_empty, FlashBank, FlashIdResult};
use crate::nvme::NvmeDevice;

const BANK_START: usize = 0x50;
const BANK_SIZE: usize = 8;
const MAX_BANKS: usize = 32;

pub fn read_flash_id(dev: &NvmeDevice) -> Result<FlashIdResult, String> {
    let mut c1_buf = [0u8; 4096];
    c1_buf[0x00] = 0x03;
    c1_buf[0x02] = 0x0C;
    c1_buf[0x04] = 0x01;
    c1_buf[0x08] = 0x08;
    c1_buf[0x0D] = 0x04;
    c1_buf[0x11] = 0x04;

    dev.admin_write(0xC1, 1, 0x400, 0, 0x08, 0, 0, 0, &c1_buf)
        .map_err(|e| format!("Tenafe C1 config write failed: {}", e))?;

    let mut c2_buf = [0u8; 4096];
    dev.admin_read(0xC2, 1, 0x400, 0, 0x08, 0, 0, 0, &mut c2_buf)
        .map_err(|e| format!("Tenafe C2 flash ID read failed: {}", e))?;

    let mut banks = Vec::new();
    for i in 0..MAX_BANKS {
        let offset = BANK_START + i * BANK_SIZE;
        if offset + BANK_SIZE > c2_buf.len() {
            break;
        }
        let bank_data = &c2_buf[offset..offset + BANK_SIZE];
        if !is_bank_empty(bank_data) {
            let mut flash_id = [0u8; 8];
            flash_id.copy_from_slice(bank_data);
            banks.push(FlashBank {
                bank_num: i as u32,
                flash_id,
            });
        }
    }

    Ok(FlashIdResult {
        controller_name: "Tenafe TC2200/TC2201".to_string(),
        banks,
    })
}
