use crate::ata::AtaDevice;
use crate::controllers::{is_bank_empty, FlashBank, FlashIdResult};

const BANK_SIZE: usize = 8;
const MAX_BANKS: usize = 16;

const MAGIC_LBA_R1: (u8, u8, u8) = (0x00, 0x00, 0xAA);
const MAGIC_LBA_R5: (u8, u8, u8) = (0x00, 0x55, 0xAA);

const SMI_FW_PREFIXES: &[(&str, &str)] = &[
    ("SM2246AA-", "SM2246EN/XT"),
    ("SM2256AB-", "SM2256/S"),
    ("SM2258AB-", "SM2258/XT"),
    ("SM2259AB-", "SM2259/XT"),
    ("SM2259AC-", "SM2259XT2"),
];

pub fn detect_from_firmware(fw: &str) -> Option<&'static str> {
    for &(prefix, name) in SMI_FW_PREFIXES {
        if fw.starts_with(prefix) {
            return Some(name);
        }
    }
    None
}

fn read_magic_lba(dev: &AtaDevice, lba_high: u8, lba_mid: u8, lba_low: u8) -> Result<[u8; 512], String> {
    let mut buf = [0u8; 512];
    dev.ata_read(0x20, 0x00, 1, lba_low, lba_mid, lba_high, 0xE0, &mut buf)?;
    Ok(buf)
}

fn read_smart_fid(dev: &AtaDevice) -> Result<[u8; 512], String> {
    let mut buf = [0u8; 512];
    dev.ata_read(0xB0, 0x00, 1, 0x00, 0x4F, 0xC2, 0x40, &mut buf)?;
    Ok(buf)
}

pub fn read_flash_id(dev: &AtaDevice) -> Result<FlashIdResult, String> {
    // Try SMART flash ID first (most reliable)
    if let Ok(fid) = read_smart_fid(dev)
        && fid.iter().any(|&b| b != 0x00 && b != 0xFF)
    {
        let mut flash_id = [0u8; 8];
        flash_id[..BANK_SIZE.min(fid.len())].copy_from_slice(&fid[..BANK_SIZE.min(fid.len())]);
        if !is_bank_empty(&flash_id) {
            return Ok(FlashIdResult {
                controller_name: "SM2259/XT (SMART FID)".to_string(),
                banks: vec![FlashBank {
                    bank_num: 0,
                    flash_id,
                }],
            });
        }
    }

    // Try magic LBA reads
    let r1 = read_magic_lba(dev, MAGIC_LBA_R1.0, MAGIC_LBA_R1.1, MAGIC_LBA_R1.2)
        .map_err(|e| format!("SMI R1 read failed: {}", e))?;

    // R5 (LBA 0x55AA) often contains flash ID data
    let r5 = read_magic_lba(dev, MAGIC_LBA_R5.0, MAGIC_LBA_R5.1, MAGIC_LBA_R5.2)
        .map_err(|e| format!("SMI R5 read failed: {}", e))?;

    // Try to extract flash IDs from R1 (primary source)
    let mut banks = Vec::new();
    for bank_num in 0..MAX_BANKS {
        let offset = bank_num * BANK_SIZE;
        if offset + BANK_SIZE > r1.len() {
            break;
        }
        let bank_data = &r1[offset..offset + BANK_SIZE];
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
            controller_name: "SM2259/XT (R1)".to_string(),
            banks,
        });
    }

    // Try R5
    banks.clear();
    for bank_num in 0..MAX_BANKS {
        let offset = bank_num * BANK_SIZE;
        if offset + BANK_SIZE > r5.len() {
            break;
        }
        let bank_data = &r5[offset..offset + BANK_SIZE];
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
            controller_name: "SM2259/XT (R5)".to_string(),
            banks,
        });
    }

    Err("no flash ID data found in SMI vendor responses".to_string())
}
