use crate::controllers::{is_bank_empty, FlashBank, FlashIdResult};
use crate::detect::RtlVariant;
use crate::nand_db::manufacturer_name;
use crate::nvme::NvmeDevice;

const BANK_SIZE: usize = 8;
const MAX_BANKS: usize = 64;

fn unlock(dev: &NvmeDevice) -> Result<(), String> {
    dev.admin_no_data(0xFC, 0, 0, 0, 0, 0x0050FFFF, 0, 0)
        .map_err(|e| format!("Realtek unlock command failed: {}", e))?;
    Ok(())
}

pub fn read_flash_id(dev: &NvmeDevice, variant: &RtlVariant) -> Result<FlashIdResult, String> {
    // Always try V1 first — V1 commands work on V2 hardware, but V2 commands
    // can hang some V2 controllers (e.g. RTS5772DL with non-standard firmware).
    let variants: &[(RtlVariant, &str)] = match variant {
        RtlVariant::V1 => &[(RtlVariant::V1, "RTS5762/63")],
        RtlVariant::V2 => &[(RtlVariant::V1, "RTS5762/63"), (RtlVariant::V2, "RTS5765/66/72")],
    };

    for &(v, ctrl_name) in variants {
        // Re-issue unlock before each attempt (vendor reads can invalidate it)
        if unlock(dev).is_err() {
            continue;
        }

        let mut buf = [0u8; 512];
        let read_ok = match v {
            RtlVariant::V1 => {
                dev.admin_read(0xFA, 0, 0x80, 0, 0, 0x00410000, 0, 0, &mut buf).is_ok()
            }
            RtlVariant::V2 => {
                dev.admin_read(0xFA, 0, 0x80, 0, 0xAFF03860, 0x00010001, 0, 0, &mut buf).is_ok()
            }
        };
        if !read_ok {
            continue;
        }

        let banks = extract_banks(&buf);
        if !banks.is_empty() {
            return Ok(FlashIdResult {
                controller_name: ctrl_name.to_string(),
                banks,
            });
        }
    }

    Err("Realtek flash ID read returned no data (tried both V1 and V2)".to_string())
}

fn extract_banks(buf: &[u8]) -> Vec<FlashBank> {
    let mut banks = Vec::new();
    for i in 0..MAX_BANKS {
        let offset = i * BANK_SIZE;
        if offset + BANK_SIZE > buf.len() {
            break;
        }
        let bank_data = &buf[offset..offset + BANK_SIZE];
        if is_bank_empty(bank_data) {
            continue;
        }
        // Filter out slots with invalid JEDEC manufacturer IDs (e.g. padding bytes)
        if manufacturer_name(bank_data[0]) == "Unknown" {
            continue;
        }
        let mut flash_id = [0u8; 8];
        flash_id.copy_from_slice(bank_data);
        banks.push(FlashBank {
            bank_num: i as u32,
            flash_id,
        });
    }
    banks
}
