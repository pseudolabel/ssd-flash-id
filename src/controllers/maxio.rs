use crate::controllers::{is_bank_empty, FlashBank, FlashIdResult};
use crate::nvme::NvmeDevice;

const MAX_CHANNELS: u8 = 8;
const MAX_CES: u8 = 8;

fn send_c1(dev: &NvmeDevice, subcmd: u8, channel: u8, ce: u8) -> Result<(), String> {
    let mut buf = [0u8; 512];
    buf[0] = 0xFF;
    buf[1] = 0xE5;
    buf[2] = subcmd;
    buf[3] = channel;
    buf[4] = ce;
    dev.admin_write(0xC1, 0, 0x80, 0, 0x001234FF, 0x01, 0, 0, &buf)
        .map_err(|e| format!("Maxio C1 write failed (subcmd 0x{:02x}): {}", subcmd, e))?;
    Ok(())
}

pub fn read_flash_id(dev: &NvmeDevice) -> Result<FlashIdResult, String> {
    send_c1(dev, 0x86, 0, 0)?;

    let mut bulk_buf = [0u8; 4096];
    dev.admin_read(0xC2, 0, 0x400, 0, 0x123486, 0x08, 0, 0, &mut bulk_buf)
        .map_err(|e| format!("Maxio C2 bulk FID read failed: {}", e))?;

    let ctrl_name = extract_controller_name(&bulk_buf);

    let mut banks = Vec::new();
    let mut bank_num = 0u32;

    for ch in 0..MAX_CHANNELS {
        for ce in 0..MAX_CES {
            if send_c1(dev, 0x36, ch, ce).is_err() {
                continue;
            }

            let mut fid_buf = [0u8; 512];
            if dev
                .admin_read(0xC2, 0, 0x80, 0, 0x123436, 0x01, 0, 0, &mut fid_buf)
                .is_err()
            {
                continue;
            }

            let bank_data = &fid_buf[..8];
            if !is_bank_empty(bank_data) {
                let mut flash_id = [0u8; 8];
                flash_id.copy_from_slice(bank_data);
                banks.push(FlashBank {
                    bank_num,
                    flash_id,
                });
            }
            bank_num += 1;
        }
    }

    Ok(FlashIdResult {
        controller_name: ctrl_name,
        banks,
    })
}

fn extract_controller_name(buf: &[u8; 4096]) -> String {
    let controller_models = [
        ",MAP1602", ",MAP1601", ",MAP1202", ",MAP1201", ",MAP1003", ",MAP1002", ",MAP1001",
    ];

    let buf_str = buf
        .iter()
        .map(|&b| {
            if b.is_ascii_graphic() || b == b',' || b == b' ' {
                b as char
            } else {
                '\0'
            }
        })
        .collect::<String>();

    for model in &controller_models {
        if let Some(pos) = buf_str.find(model) {
            return buf_str[pos + 1..pos + 1 + model.len() - 1].to_string();
        }
    }

    "Maxio MAP".to_string()
}
