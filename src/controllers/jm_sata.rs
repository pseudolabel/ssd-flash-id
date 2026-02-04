use crate::ata::AtaDevice;
use crate::controllers::{is_bank_empty, FlashBank, FlashIdResult};

const ATA_CMD_WRITE: u8 = 0x88;
const ATA_CMD_READ: u8 = 0x86;
const ATA_DEVICE: u8 = 0xE0;
const ATA_COUNT: u8 = 0x34;
const ATA_FEATURES_WRITE: u8 = 0x12;
const ATA_LBA_LOW_WRITE: u8 = 0xFF;
const ATA_LBA_LOW_READ: u8 = 0x12;

const SUBCMD_FW_ID: u8 = 0x86;
const SUBCMD_PER_CHANNEL_FID: u8 = 0x36;

const FW_RESPONSE_SIZE: usize = 4096;
const MAX_LOOKUP_ENTRIES: usize = 32;
const FLASH_ID_LEN: usize = 7;

const MAS1102_TABLE_OFFSET: usize = 0x474;
const MAS1102_CE_SHIFT: u8 = 3;
const MAS1102_NAND_OFFSET: usize = 0x894;
const MAS1102_NAND_MAX_LEN: usize = 80;

const MAS0902_TABLE_OFFSET: usize = 0x450;
const MAS0902_CE_SHIFT: u8 = 4;
const MAS0902_NAND_OFFSET: usize = 0x568;
const MAS0902_NAND_MAX_LEN: usize = 32;

const JMF_NAND_OFFSET: usize = 0x4A8;
const JMF_NAND_MAX_LEN: usize = 16;

const CONTROLLER_PATTERNS: &[(&str, &str)] = &[
    (",MA1102", "MAS1102"),
    (",DM1102", "MAS1102"),
    (",MK8215", "MAS0902"),
    (",DM9343", "MAS0902"),
    (",805", "MK8115"),
    (",670", "JMF670"),
    (",667", "JMF667"),
    (",662", "JMF662"),
    (",661", "JMF661"),
    (",61X", "JMF612"),
    (",608", "JMF608"),
    (",607", "JMF607"),
    (",606", "JMF606"),
    (",605", "JMF605"),
];

#[derive(Debug, Clone)]
pub enum JmController {
    Mas1102,
    Mas0902,
    Jmf(String),
}

impl JmController {
    fn display_name(&self) -> &str {
        match self {
            JmController::Mas1102 => "MAS1102",
            JmController::Mas0902 => "MAS0902",
            JmController::Jmf(name) => name,
        }
    }
}

fn unlock(dev: &AtaDevice) -> Result<(), String> {
    // Generic unlock: SET FEATURES (0xEF), features=0xDA, count=0x41
    let _ = dev.ata_no_data(0xEF, 0xDA, 0x41, 0x00, 0x00, 0x00, ATA_DEVICE);
    // Fallback unlock: SET FEATURES (0xEF), features=0xDC, count=0x4A
    let _ = dev.ata_no_data(0xEF, 0xDC, 0x4A, 0x00, 0x00, 0x00, ATA_DEVICE);
    Ok(())
}

pub fn read_firmware_id(dev: &AtaDevice) -> Result<Vec<u8>, String> {
    unlock(dev)?;

    // Try jm_id variant (sub-command 0x86) with sector count matching buffer size
    if let Ok(buf) = try_read_firmware_id(dev, SUBCMD_FW_ID, 0x03)
        && buf.iter().any(|&b| b != 0)
    {
        return Ok(buf);
    }

    // Try jm_fid variant (sub-command 0x04)
    if let Ok(buf) = try_read_firmware_id(dev, 0x04, 0xFF)
        && buf.iter().any(|&b| b != 0)
    {
        return Ok(buf);
    }

    Err("firmware ID response is all zeros (controller may not support JMicron/Maxio vendor commands)".to_string())
}

fn try_read_firmware_id(dev: &AtaDevice, subcmd: u8, param: u8) -> Result<Vec<u8>, String> {
    let mut payload = [0u8; 512];
    payload[0] = 0xFF;
    payload[1] = 0xE5;
    payload[2] = subcmd;
    payload[3] = param;

    // Write uses 1 sector (512 bytes), read uses 8 sectors (4096 bytes)
    dev.ata_write(
        ATA_CMD_WRITE,
        ATA_FEATURES_WRITE,
        1,
        ATA_LBA_LOW_WRITE,
        0x00,
        0x00,
        ATA_DEVICE,
        &payload,
    )
    .map_err(|e| format!("JM firmware ID write failed: {}", e))?;

    let mut buf = vec![0u8; FW_RESPONSE_SIZE];
    dev.ata_read(
        ATA_CMD_READ,
        subcmd,
        8,
        ATA_LBA_LOW_READ,
        0x00,
        0x00,
        ATA_DEVICE,
        &mut buf,
    )
    .map_err(|e| format!("JM firmware ID read failed: {}", e))?;

    Ok(buf)
}

pub fn detect(fw_response: &[u8]) -> Option<JmController> {
    let buf_str: String = fw_response
        .iter()
        .map(|&b| {
            if b.is_ascii_graphic() || b == b',' || b == b' ' {
                b as char
            } else {
                '\0'
            }
        })
        .collect();

    for &(pattern, name) in CONTROLLER_PATTERNS {
        if buf_str.contains(pattern) {
            return match name {
                "MAS1102" => Some(JmController::Mas1102),
                "MAS0902" => Some(JmController::Mas0902),
                _ => Some(JmController::Jmf(name.to_string())),
            };
        }
    }

    None
}

pub fn read_flash_id(dev: &AtaDevice, fw_response: &[u8]) -> Result<FlashIdResult, String> {
    let controller = detect(fw_response)
        .ok_or_else(|| "unable to detect JMicron/Maxio controller from firmware response".to_string())?;

    let (table_offset, ce_shift) = match controller {
        JmController::Mas1102 => (MAS1102_TABLE_OFFSET, MAS1102_CE_SHIFT),
        JmController::Mas0902 => (MAS0902_TABLE_OFFSET, MAS0902_CE_SHIFT),
        JmController::Jmf(_) => {
            return Err("per-channel flash ID reading requires MAS1102 or MAS0902".to_string());
        }
    };

    let ctrl_name = build_controller_name(&controller, fw_response);

    let mut banks = Vec::new();
    let mut bank_num = 0u32;

    for i in 0..MAX_LOOKUP_ENTRIES {
        let offset = table_offset + i;
        if offset >= fw_response.len() {
            break;
        }

        let entry = fw_response[offset];
        if entry == 0xFF {
            continue;
        }

        let channel = entry >> ce_shift;

        if let Ok(flash_id) = read_channel_fid(dev, channel, entry)
            && !is_bank_empty(&flash_id)
        {
            banks.push(FlashBank {
                bank_num,
                flash_id,
            });
        }
        bank_num += 1;
    }

    Ok(FlashIdResult {
        controller_name: ctrl_name,
        banks,
    })
}

fn read_channel_fid(dev: &AtaDevice, channel: u8, raw_entry: u8) -> Result<[u8; 8], String> {
    let mut payload = [0u8; 512];
    payload[0] = 0xFF;
    payload[1] = 0xE5;
    payload[2] = SUBCMD_PER_CHANNEL_FID;
    payload[7] = channel;
    payload[0x1C] = raw_entry;

    dev.ata_write(
        ATA_CMD_WRITE,
        ATA_FEATURES_WRITE,
        ATA_COUNT,
        ATA_LBA_LOW_WRITE,
        0x00,
        0x00,
        ATA_DEVICE,
        &payload,
    )
    .map_err(|e| format!("JM per-channel FID write failed (ch {}): {}", channel, e))?;

    let mut buf = [0u8; 512];
    dev.ata_read(
        ATA_CMD_READ,
        SUBCMD_PER_CHANNEL_FID,
        ATA_COUNT,
        ATA_LBA_LOW_READ,
        0x00,
        0x00,
        ATA_DEVICE,
        &mut buf,
    )
    .map_err(|e| format!("JM per-channel FID read failed (ch {}): {}", channel, e))?;

    let mut flash_id = [0u8; 8];
    flash_id[..FLASH_ID_LEN].copy_from_slice(&buf[..FLASH_ID_LEN]);
    Ok(flash_id)
}

fn build_controller_name(controller: &JmController, fw_response: &[u8]) -> String {
    let nand_str = extract_nand_string(controller, fw_response);
    let base = controller.display_name();
    if nand_str.is_empty() {
        base.to_string()
    } else {
        format!("{} ({})", base, nand_str)
    }
}

fn extract_nand_string(controller: &JmController, fw_response: &[u8]) -> String {
    let (offset, max_len) = match controller {
        JmController::Mas1102 => (MAS1102_NAND_OFFSET, MAS1102_NAND_MAX_LEN),
        JmController::Mas0902 => (MAS0902_NAND_OFFSET, MAS0902_NAND_MAX_LEN),
        JmController::Jmf(_) => (JMF_NAND_OFFSET, JMF_NAND_MAX_LEN),
    };

    if offset >= fw_response.len() {
        return String::new();
    }

    let end = (offset + max_len).min(fw_response.len());
    let slice = &fw_response[offset..end];

    let s: String = slice
        .iter()
        .take_while(|&&b| (0x20..=0x7A).contains(&b))
        .map(|&b| b as char)
        .collect();

    s.trim().to_string()
}
