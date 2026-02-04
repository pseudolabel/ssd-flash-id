use crate::controllers::{is_bank_empty, FlashBank, FlashIdResult};
use crate::nvme::NvmeDevice;

const GRIT_MAGIC: u32 = 0x54495247;
const INNO_MAGIC: u32 = 0x4F4E4E49;

const DID_OFFSET: usize = 0x62E;
const FID_OFFSET_5208: usize = 0x548;
const FID_OFFSET_5220: usize = 0x24E;
const MAX_BANKS_5208: usize = 32;
const MAX_BANKS_5220: usize = 64;
const FID_ENTRY_SIZE: usize = 6;

pub fn read_flash_id(dev: &NvmeDevice) -> Result<FlashIdResult, String> {
    let mut buf = [0u8; 4096];
    dev.admin_read(
        0xF2, 0, 0x400, 0, 0, 0, GRIT_MAGIC, INNO_MAGIC, &mut buf,
    )
    .map_err(|e| format!("Innogrit vendor command failed: {}", e))?;

    let did = u16::from_le_bytes([buf[DID_OFFSET], buf[DID_OFFSET + 1]]);
    let ctrl_name = format!("IG{}", did);

    let (fid_offset, max_banks) = match did {
        0x5208 | 0x5216 => (FID_OFFSET_5208, MAX_BANKS_5208),
        _ => (FID_OFFSET_5220, MAX_BANKS_5220),
    };

    let mut banks = Vec::new();
    for i in 0..max_banks {
        let offset = fid_offset + i * FID_ENTRY_SIZE;
        if offset + FID_ENTRY_SIZE > buf.len() {
            break;
        }
        let entry = &buf[offset..offset + FID_ENTRY_SIZE];
        if is_bank_empty(entry) {
            continue;
        }
        let mut flash_id = [0u8; 8];
        flash_id[..FID_ENTRY_SIZE].copy_from_slice(entry);
        banks.push(FlashBank {
            bank_num: i as u32,
            flash_id,
        });
    }

    Ok(FlashIdResult {
        controller_name: ctrl_name,
        banks,
    })
}
