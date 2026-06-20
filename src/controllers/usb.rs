use crate::ata::AtaDevice;
use crate::controllers::{FlashBank, FlashIdResult};

const MAX_FLASH_IDS: usize = 8;

#[derive(Clone, PartialEq, Eq)]
struct RawFlashId {
    channel: u32,
    id: Vec<u8>,
}

pub fn read_flash_id(dev: &AtaDevice, family: &str) -> Result<FlashIdResult, String> {
    match family {
        "cbm2199" => detect_cbm2199(dev),
        "fc2279" => detect_fc2279(dev),
        "fc3379" => detect_fc3379(dev),
        "au8910x" => detect_au8910x(dev),
        "au6989" => detect_au6989(dev),
        _ => Err(format!("unknown USB controller type '{}'", family)),
    }
}

fn detect_cbm2199(dev: &AtaDevice) -> Result<FlashIdResult, String> {
    const VARIANTS: &[(&[u8], &str)] = &[
        (b"ChipsBank2199EB", "ChipsBank CBM2199EB"),
        (b"ChipsBank2199ES", "ChipsBank CBM2199ES"),
        (b"ChipsBank2199E", "ChipsBank CBM2199E"),
        (b"ChipsBank2199SC", "ChipsBank CBM2199SC"),
        (b"ChipsBank2199S", "ChipsBank CBM2199S"),
        (b"ChipsBank2199C", "ChipsBank CBM2199C"),
        (b"ChipsBank2199", "ChipsBank CBM2199"),
    ];

    let mut info = vec![0u8; 0x600];
    dev.read_scsi(
        &[
            0xea, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xe3,
        ],
        &mut info,
    )?;
    let controller = VARIANTS
        .iter()
        .find_map(|(needle, name)| contains(&info, needle).then_some(*name))
        .ok_or_else(|| "CBM2199 controller-info response was not recognized".to_string())?;

    let mut blob = vec![0u8; 0x800];
    let cdbs: [&[u8]; 2] = [
        &[
            0xea, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xe6,
        ],
        &[
            0xea, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xe6,
        ],
    ];
    read_first(dev, &cdbs, &mut blob)?;

    let mut ids = Vec::new();
    add_flash_id(&mut ids, 0, &blob[0x684..0x684 + 6]);
    result(controller, ids)
}

fn detect_fc2279(dev: &AtaDevice) -> Result<FlashIdResult, String> {
    let mut flash_info = vec![0u8; 0x800];
    let cdbs: [&[u8]; 2] = [
        &[
            0xf1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xa1,
        ],
        &[
            0xf1, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xa1,
        ],
    ];
    if read_first(dev, &cdbs, &mut flash_info).is_ok()
        && flash_info[8] == 0x01
        && flash_info[9] == 0x01
    {
        let mut ids = Vec::new();
        add_flash_id(&mut ids, 0, &flash_info[0x10..0x10 + 6]);
        if !ids.is_empty() {
            return result("FirstChip FC2279", ids);
        }
    }

    let mut config = vec![0u8; 0x4000];
    dev.read_scsi(
        &[
            0xf1, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xaa,
        ],
        &mut config,
    )?;
    let mut ids = Vec::new();
    add_flash_id(&mut ids, 0, &config[0x12d..0x12d + 6]);
    result("FirstChip FC2279", ids)
}

fn detect_fc3379(dev: &AtaDevice) -> Result<FlashIdResult, String> {
    const VARIANTS: &[(&[u8], &str)] = &[
        (b"FC3281C", "FirstChip FC3379"),
        (b"FC3281", "FirstChip FC3379"),
    ];

    let mut info = vec![0u8; 0x200];
    dev.read_scsi(
        &[
            0xf1, 0x00, 0x00, 0x40, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xa0,
        ],
        &mut info,
    )?;
    let controller = VARIANTS
        .iter()
        .find_map(|(needle, name)| contains(&info, needle).then_some(*name))
        .ok_or_else(|| "FC3379 controller-info response was not recognized".to_string())?;

    fc3379_prepare(dev)?;

    let mut table = vec![0u8; 0x100];
    let cdbs: [&[u8]; 2] = [
        &[
            0xf1, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xa1,
        ],
        &[
            0xf1, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xa1,
        ],
    ];
    read_first(dev, &cdbs, &mut table)?;
    result(controller, fc3379_flash_ids(&table))
}

fn fc3379_prepare(dev: &AtaDevice) -> Result<(), String> {
    let mut empty = [];
    dev.read_scsi(
        &[
            0xf1, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xa9,
        ],
        &mut empty,
    )?;

    let mut scratch = [0u8; 4];
    for cdb in [
        &[
            0xf1, 0x08, 0x00, 0x00, 0x40, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xb1,
        ],
        &[
            0xf1, 0x30, 0x00, 0x00, 0x40, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xb1,
        ],
    ] {
        dev.read_scsi(cdb, &mut scratch)?;
    }

    Ok(())
}

fn detect_au8910x(dev: &AtaDevice) -> Result<FlashIdResult, String> {
    let mut table = vec![0u8; 0x200];
    dev.read_scsi(
        &[0xfa, 0x17, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00],
        &mut table,
    )?;
    result("Alcor AU89103", alcor_flash_ids(&table))
}

fn detect_au6989(dev: &AtaDevice) -> Result<FlashIdResult, String> {
    let mut controller = "Alcor AU6989";
    let mut info = vec![0u8; 0x200];
    if dev
        .read_scsi(&[0xfa, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], &mut info)
        .is_ok()
    {
        controller = match info.get(0x14).copied() {
            Some(0x11) => "Alcor AU6989SN",
            Some(0x22) => "Alcor AU6989SN-GTC",
            _ => "Alcor AU6989",
        };
    }

    let mut table = vec![0u8; 0x200];
    dev.read_scsi(
        &[0xfa, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        &mut table,
    )?;
    result(controller, alcor_flash_ids(&table))
}

fn read_first(dev: &AtaDevice, cdbs: &[&[u8]], buf: &mut [u8]) -> Result<(), String> {
    let mut last = String::new();
    for cdb in cdbs {
        buf.fill(0);
        match dev.read_scsi(cdb, buf) {
            Ok(()) => return Ok(()),
            Err(e) => last = e,
        }
    }
    Err(last)
}

fn result(controller_name: &str, ids: Vec<RawFlashId>) -> Result<FlashIdResult, String> {
    let banks: Vec<FlashBank> = ids
        .into_iter()
        .map(|raw| {
            let mut flash_id = [0u8; 8];
            let len = raw.id.len().min(flash_id.len());
            flash_id[..len].copy_from_slice(&raw.id[..len]);
            FlashBank {
                bank_num: raw.channel,
                flash_id,
            }
        })
        .collect();
    if banks.is_empty() {
        return Err(format!(
            "{controller_name} response did not contain a flash ID"
        ));
    }
    Ok(FlashIdResult {
        controller_name: controller_name.to_string(),
        banks,
    })
}

fn add_flash_id(ids: &mut Vec<RawFlashId>, channel: u32, id: &[u8]) {
    if !flash_id_valid(id)
        || ids
            .iter()
            .any(|fid| fid.channel == channel && fid.id.as_slice() == id)
        || ids.len() >= MAX_FLASH_IDS
    {
        return;
    }
    ids.push(RawFlashId {
        channel,
        id: id.to_vec(),
    });
}

fn fc3379_flash_ids(buf: &[u8]) -> Vec<RawFlashId> {
    let mut ids = Vec::new();
    if buf.len() < 0x18 || buf[8] as usize > MAX_FLASH_IDS {
        return ids;
    }
    for i in 0..(buf[8] as usize).min(MAX_FLASH_IDS) {
        let offset = 0x10 + i * 0x10;
        if offset + 8 > buf.len() {
            break;
        }
        add_flash_id(&mut ids, i as u32, &buf[offset..offset + 8]);
    }
    ids
}

fn alcor_flash_ids(buf: &[u8]) -> Vec<RawFlashId> {
    let mut ids = Vec::new();
    for i in 0..MAX_FLASH_IDS {
        let offset = i * 0x10;
        if offset + 6 > buf.len() {
            break;
        }
        add_flash_id(&mut ids, i as u32, &buf[offset..offset + 6]);
    }
    ids
}

fn flash_id_valid(id: &[u8]) -> bool {
    !id.is_empty()
        && !is_repeated(id, 0x00)
        && !is_repeated(id, 0xff)
        && !flash_id_has_padding_tail(id)
}

fn flash_id_has_padding_tail(id: &[u8]) -> bool {
    id.len() > 1 && (is_repeated(&id[1..], 0x00) || is_repeated(&id[1..], 0xff))
}

fn is_repeated(buf: &[u8], value: u8) -> bool {
    buf.iter().all(|&b| b == value)
}

fn contains(buf: &[u8], needle: &[u8]) -> bool {
    buf.windows(needle.len()).any(|w| w == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_flash_ids() {
        assert!(!flash_id_valid(&[]));
        assert!(!flash_id_valid(&[0, 0, 0, 0, 0, 0]));
        assert!(!flash_id_valid(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff]));
        assert!(!flash_id_valid(&[0x98, 0, 0, 0, 0, 0]));
        assert!(flash_id_valid(&[0x98, 0xde, 0x94, 0x93, 0x76, 0x51]));
    }

    #[test]
    fn deduplicates_channel_and_id() {
        let mut ids = Vec::new();
        add_flash_id(&mut ids, 0, &[0x98, 0xde, 0x94, 0x93, 0x76, 0x51]);
        add_flash_id(&mut ids, 0, &[0x98, 0xde, 0x94, 0x93, 0x76, 0x51]);
        add_flash_id(&mut ids, 1, &[0x98, 0xde, 0x94, 0x93, 0x76, 0x51]);
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn parses_fc3379_table() {
        let mut buf = [0u8; 0x100];
        buf[8] = 2;
        buf[0x10..0x18].copy_from_slice(&[0x98, 0xde, 0x94, 0x93, 0x76, 0x51, 0x08, 0x1e]);
        buf[0x20..0x28].copy_from_slice(&[0xec, 0x3a, 0x94, 0x93, 0x76, 0x51, 0x08, 0x1e]);
        let ids = fc3379_flash_ids(&buf);
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[1].channel, 1);
        assert_eq!(ids[0].id, vec![0x98, 0xde, 0x94, 0x93, 0x76, 0x51, 0x08, 0x1e]);
    }

    #[test]
    fn parses_alcor_table() {
        let mut buf = [0u8; 0x200];
        buf[0..6].copy_from_slice(&[0x98, 0xde, 0x94, 0x93, 0x76, 0x51]);
        buf[0x10..0x16].copy_from_slice(&[0xec, 0x3a, 0x94, 0x93, 0x76, 0x51]);
        let ids = alcor_flash_ids(&buf);
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0].id, vec![0x98, 0xde, 0x94, 0x93, 0x76, 0x51]);
    }

    #[test]
    fn cbm2199_variant_order_is_specific_first() {
        let mut info = vec![0u8; 0x600];
        info[8..22].copy_from_slice(b"ChipsBank2199E");
        let variants: &[(&[u8], &str)] = &[
            (b"ChipsBank2199EB", "ChipsBank CBM2199EB"),
            (b"ChipsBank2199ES", "ChipsBank CBM2199ES"),
            (b"ChipsBank2199E", "ChipsBank CBM2199E"),
            (b"ChipsBank2199", "ChipsBank CBM2199"),
        ];
        let name = variants
            .iter()
            .find_map(|(needle, name)| contains(&info, needle).then_some(*name))
            .unwrap();
        assert_eq!(name, "ChipsBank CBM2199E");
    }
}
