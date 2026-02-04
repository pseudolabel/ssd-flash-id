use crate::ata::AtaDevice;
use crate::controllers::{is_bank_empty, FlashBank, FlashIdResult};

const BANK_SIZE: usize = 8;
const MAX_BANKS: usize = 8;

const RTL_FW_PREFIXES: &[(&str, &str)] = &[
    ("REALTEK_RL6468", "RTS5732"),
    ("REALTEK_RL6531", "RTS5733"),
    ("REALTEK_RL6643", "RTS5735"),
];

pub fn detect_from_firmware(fw: &str) -> Option<&'static str> {
    for &(prefix, name) in RTL_FW_PREFIXES {
        if fw.starts_with(prefix) {
            return Some(name);
        }
    }
    None
}

fn fid_prefix(dev: &AtaDevice) -> Result<(), String> {
    // Setup command: 0xFC, Features=0x50, Count=0xFF, Device=0xE0
    // PrevCount=0xFF, all other prev=0
    dev.ata_no_data_ext(
        0xFC, 0x50, 0xFF, 0x00, 0x00, 0x00, 0xE0,
        0x00, 0xFF, 0x00, 0x00, 0x00,
    )
}

fn read_fid(dev: &AtaDevice) -> Result<[u8; 512], String> {
    let mut buf = [0u8; 512];
    // Get FID: Command=0xFA, Features=0x01, Count=1, LBA=0xF00420, Device=0xE0
    // PrevLBA_Low=0xAF
    dev.ata_read_ext(
        0xFA, 0x01, 0x01, 0x20, 0x04, 0xF0, 0xE0,
        0x00, 0x00, 0xAF, 0x00, 0x00,
        &mut buf,
    )?;
    Ok(buf)
}

fn read_fid2(dev: &AtaDevice) -> Result<[u8; 512], String> {
    let mut buf = [0u8; 512];
    // Get FID2: Command=0xFA, Features=0x41, Count=1, all LBA=0, Device=0xE0
    dev.ata_read_ext(
        0xFA, 0x41, 0x01, 0x00, 0x00, 0x00, 0xE0,
        0x00, 0x00, 0x00, 0x00, 0x00,
        &mut buf,
    )?;
    Ok(buf)
}

fn extract_banks(data: &[u8; 512]) -> Vec<FlashBank> {
    let mut banks = Vec::new();
    for bank_num in 0..MAX_BANKS {
        let offset = bank_num * BANK_SIZE;
        if offset + BANK_SIZE > data.len() {
            break;
        }
        let bank_data = &data[offset..offset + BANK_SIZE];
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
    banks
}

pub fn read_flash_id(dev: &AtaDevice) -> Result<FlashIdResult, String> {
    // Send FID prefix/setup command
    fid_prefix(dev).map_err(|e| format!("Realtek FID prefix failed: {}", e))?;

    // Try primary flash ID read
    if let Ok(fid) = read_fid(dev) {
        let banks = extract_banks(&fid);
        if !banks.is_empty() {
            return Ok(FlashIdResult {
                controller_name: "Realtek SATA".to_string(),
                banks,
            });
        }
    }

    // Try extended flash ID
    if let Ok(fid2) = read_fid2(dev) {
        let banks = extract_banks(&fid2);
        if !banks.is_empty() {
            return Ok(FlashIdResult {
                controller_name: "Realtek SATA (ext)".to_string(),
                banks,
            });
        }
    }

    Err("no flash ID data found in Realtek SATA vendor responses".to_string())
}
