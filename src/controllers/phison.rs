use crate::controllers::{is_bank_empty, FlashBank, FlashIdResult};
use crate::nvme::NvmeDevice;

const PHISON_SIGNATURE: &[u8; 8] = b"PhIsOnNo";
const MAX_BANKS: u32 = 8;

fn phison_crc(buf: &[u8; 64]) -> u32 {
    let mut crc: u16 = 0;
    for &b in &buf[..60] {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
            crc &= 0xFFFF;
        }
    }
    let swapped = (crc >> 8) | ((crc & 0xFF) << 8);
    (swapped as u32) << 16
}

fn build_cmd(opcode: u8, cdw10: u32, cdw12: u32, cdw13: u32) -> u32 {
    let mut buf = [0u8; 64];
    buf[0] = opcode;
    buf[0x28..0x2C].copy_from_slice(&cdw10.to_le_bytes());
    buf[0x30..0x34].copy_from_slice(&cdw12.to_le_bytes());
    buf[0x34..0x38].copy_from_slice(&cdw13.to_le_bytes());
    phison_crc(&buf)
}

pub fn read_flash_id(dev: &NvmeDevice) -> Result<FlashIdResult, String> {
    let cdw15 = build_cmd(0xD2, 0x400, 0x80, 0);
    let mut sysinfo = [0u8; 4096];
    dev.admin_read(0xD2, 0, 0x400, 0, 0x80, 0, 0, cdw15, &mut sysinfo)
        .map_err(|e| format!("Phison system info read failed: {}", e))?;

    let is_phison = sysinfo
        .windows(PHISON_SIGNATURE.len())
        .any(|w| w == PHISON_SIGNATURE);
    if !is_phison {
        return Err("Phison signature 'PhIsOnNo' not found in system info".to_string());
    }

    let ctrl_name = extract_controller_name(&sysinfo);

    let mut banks = Vec::new();

    if let Some(extracted) = extract_sysinfo_flash_ids(&sysinfo) {
        banks = extracted;
    }

    if banks.is_empty() {
        for bank in 0..MAX_BANKS {
            let cdw12 = (bank << 8) | 0x90;
            let cdw15_bank = build_cmd(0xD2, 0x80, cdw12, 0);
            let mut buf = [0u8; 512];
            if dev
                .admin_read(0xD2, 0, 0x80, 0, cdw12, 0, 0, cdw15_bank, &mut buf)
                .is_ok()
            {
                let bank_data = &buf[..8];
                if !is_bank_empty(bank_data) {
                    let mut flash_id = [0u8; 8];
                    flash_id.copy_from_slice(bank_data);
                    banks.push(FlashBank {
                        bank_num: bank,
                        flash_id,
                    });
                }
            }
        }
    }

    Ok(FlashIdResult {
        controller_name: ctrl_name,
        banks,
    })
}

fn extract_controller_name(sysinfo: &[u8; 4096]) -> String {
    for window in sysinfo.windows(6) {
        if window[0] == b'P' && window[1] == b'S' && window[2] == b'5' && window[3] == b'0'
            && let Some(end) = sysinfo[window.as_ptr() as usize - sysinfo.as_ptr() as usize..]
                .iter()
                .position(|&b| b == 0 || b == b' ' || !b.is_ascii_graphic())
        {
            let start = window.as_ptr() as usize - sysinfo.as_ptr() as usize;
            let name = &sysinfo[start..start + end.min(16)];
            let s: String = name.iter().map(|&b| b as char).collect();
            if !s.is_empty() {
                return s;
            }
        }
    }

    for window in sysinfo.windows(4) {
        if window[0] == b'P' && window[1] == b'S' && window[2] == b'3' && window[3] == b'1' {
            let start = window.as_ptr() as usize - sysinfo.as_ptr() as usize;
            let end = sysinfo[start..]
                .iter()
                .position(|&b| b == 0 || b == b' ' || !b.is_ascii_graphic())
                .unwrap_or(16)
                .min(16);
            let name = &sysinfo[start..start + end];
            let s: String = name.iter().map(|&b| b as char).collect();
            if !s.is_empty() {
                return s;
            }
        }
    }

    "Phison".to_string()
}

fn extract_sysinfo_flash_ids(sysinfo: &[u8; 4096]) -> Option<Vec<FlashBank>> {
    let mut banks = Vec::new();

    // E12+ controllers embed flash IDs in the 4KB system info response.
    // Scan for 8-byte flash ID entries at likely offsets in the sysinfo.
    // The flash IDs are typically in the 0x70-0x8B region with spacing.
    for start_offset in [0x70usize, 0x78, 0x80, 0x88, 0x90, 0x98, 0xA0] {
        if start_offset + 8 > sysinfo.len() {
            continue;
        }
        let candidate = &sysinfo[start_offset..start_offset + 8];
        if !is_bank_empty(candidate) && is_known_manufacturer(candidate[0]) {
            let mut flash_id = [0u8; 8];
            flash_id.copy_from_slice(candidate);
            let already_have = banks.iter().any(|b: &FlashBank| b.flash_id == flash_id);
            if !already_have {
                banks.push(FlashBank {
                    bank_num: banks.len() as u32,
                    flash_id,
                });
            }
        }
    }

    if banks.is_empty() {
        None
    } else {
        Some(banks)
    }
}

fn is_known_manufacturer(id: u8) -> bool {
    matches!(
        id,
        0x01 | 0x04
            | 0x07
            | 0x20
            | 0x2C
            | 0x45
            | 0x4A
            | 0x51
            | 0x89
            | 0x92
            | 0x98
            | 0x9B
            | 0xAD
            | 0xB5
            | 0xC2
            | 0xC8
            | 0xEC
            | 0xEF
    )
}
