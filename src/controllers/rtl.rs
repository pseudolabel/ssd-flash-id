use crate::controllers::{is_bank_empty, FlashBank, FlashIdResult};
use crate::detect::RtlVariant;
use crate::nvme::NvmeDevice;

const BANK_SIZE: usize = 8;
const MAX_BANKS: usize = 64;

pub fn read_flash_id(dev: &NvmeDevice, variant: &RtlVariant) -> Result<FlashIdResult, String> {
    dev.admin_no_data(0xFC, 0, 0, 0, 0, 0x0050FFFF, 0, 0)
        .map_err(|e| format!("Realtek unlock command failed: {}", e))?;

    let mut buf = [0u8; 512];
    match variant {
        RtlVariant::V1 => {
            dev.admin_read(0xFA, 0, 0x80, 0, 0, 0x00410000, 0, 0, &mut buf)
                .map_err(|e| format!("Realtek V1 flash ID read failed: {}", e))?;
        }
        RtlVariant::V2 => {
            dev.admin_read(0xFA, 0, 0x80, 0, 0xAFF03860, 0x00010001, 0, 0, &mut buf)
                .map_err(|e| format!("Realtek V2 flash ID read failed: {}", e))?;
        }
    }

    let ctrl_name = match variant {
        RtlVariant::V1 => "RTS5762/63",
        RtlVariant::V2 => "RTS5765/66/72",
    };

    let mut banks = Vec::new();
    for i in 0..MAX_BANKS {
        let offset = i * BANK_SIZE;
        if offset + BANK_SIZE > buf.len() {
            break;
        }
        let bank_data = &buf[offset..offset + BANK_SIZE];
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
        controller_name: ctrl_name.to_string(),
        banks,
    })
}
