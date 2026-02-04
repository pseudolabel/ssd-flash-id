use crate::controllers::{is_bank_empty, FlashBank, FlashIdResult};
use crate::nvme::NvmeDevice;

const DM1160_SIG: &[u8; 6] = b"DM1160";
const DM1140_SIG: &[u8; 6] = b"DM1140";

pub fn read_flash_id(dev: &NvmeDevice) -> Result<FlashIdResult, String> {
    let mut fw_buf = [0u8; 512];
    dev.admin_read(0xFE, 0, 0x80, 0, 0, 0, 0, 0xA1, &mut fw_buf)
        .map_err(|e| format!("Marvell firmware info read failed: {}", e))?;

    if &fw_buf[..6] != DM1160_SIG && &fw_buf[..6] != DM1140_SIG {
        return Err(format!(
            "not a Marvell 88NV1160/1140 controller (got {:?})",
            &fw_buf[..6]
        ));
    }

    let ctrl_name = extract_fw_name(&fw_buf);

    let mut req_buf = [0u8; 512];
    req_buf[0] = 0x01;
    dev.admin_write(0xFD, 0, 0x80, 0, 0, 0, 0x6299, 0x50, &req_buf)
        .map_err(|e| format!("Marvell flash ID request failed: {}", e))?;

    let mut fid_buf = [0u8; 1024];
    dev.admin_read(0xFE, 0, 0x100, 0, 0, 0, 0x6299, 0x50, &mut fid_buf)
        .map_err(|e| format!("Marvell flash ID read failed: {}", e))?;

    let mut banks = Vec::new();
    for i in 0..64usize {
        let offset = i * 8;
        if offset + 8 > fid_buf.len() {
            break;
        }
        let bank_data = &fid_buf[offset..offset + 8];
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
        controller_name: ctrl_name,
        banks,
    })
}

fn extract_fw_name(buf: &[u8; 512]) -> String {
    let end = buf
        .iter()
        .position(|&b| b == 0 || (!b.is_ascii_graphic() && b != b' ' && b != b'-'))
        .unwrap_or(64)
        .min(64);
    let s: String = buf[..end]
        .iter()
        .map(|&b| if b.is_ascii_graphic() || b == b' ' || b == b'-' { b as char } else { ' ' })
        .collect();
    let trimmed = s.trim();
    if trimmed.is_empty() {
        "Marvell 88NV1160".to_string()
    } else {
        trimmed.to_string()
    }
}
