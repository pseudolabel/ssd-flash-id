use crate::controllers::{is_bank_empty, FlashBank, FlashIdResult};

// Known JEDEC manufacturer IDs for NAND flash
const JEDEC_IDS: &[u8] = &[
    0x01, // Spansion
    0x04, // Fujitsu
    0x07, // Renesas
    0x20, // STMicro
    0x2C, // Micron
    0x45, // SanDisk/WD
    0x4A, // SMIC
    0x89, // Intel
    0x92, // ESMT/PowerChip
    0x98, // Kioxia/Toshiba
    0x9B, // YMTC
    0xAD, // SK Hynix
    0xB5, // Spectek
    0xC2, // Macronix
    0xC8, // ESMT/MIRA-PSC
    0xEC, // Samsung
    0xEF, // Winbond
];

// Some controllers embed NAND flash IDs in ATA IDENTIFY DEVICE vendor-specific words.
// Yeestor/SiliconGo stores 8 bytes at offset 0x127-0x12E (words 147-151, no byte-swap).
const ATA_ID_FID_OFFSET: usize = 0x127;
const ATA_ID_FID_LEN: usize = 8;

pub fn extract_from_identify(id_data: &[u8; 512]) -> Option<FlashIdResult> {
    if ATA_ID_FID_OFFSET + ATA_ID_FID_LEN > id_data.len() {
        return None;
    }

    let fid = &id_data[ATA_ID_FID_OFFSET..ATA_ID_FID_OFFSET + ATA_ID_FID_LEN];

    if is_bank_empty(fid) {
        return None;
    }

    // Validate manufacturer ID
    if !JEDEC_IDS.contains(&fid[0]) {
        return None;
    }

    let mut flash_id = [0u8; 8];
    flash_id.copy_from_slice(fid);

    Some(FlashIdResult {
        controller_name: "SATA (from ATA IDENTIFY)".to_string(),
        banks: vec![FlashBank {
            bank_num: 0,
            flash_id,
        }],
    })
}
