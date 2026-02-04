use crate::nvme::NvmeDevice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RtlVariant {
    V1,
    V2,
}

#[derive(Debug, Clone)]
pub enum ControllerType {
    Smi(String),
    Realtek(String, RtlVariant),
    Phison(String),
    Maxio(String),
    Marvell(String),
    Innogrit(String),
    Tenafe(String),
}

impl ControllerType {
    pub fn name(&self) -> &str {
        match self {
            ControllerType::Smi(n)
            | ControllerType::Realtek(n, _)
            | ControllerType::Phison(n)
            | ControllerType::Maxio(n)
            | ControllerType::Marvell(n)
            | ControllerType::Innogrit(n)
            | ControllerType::Tenafe(n) => n,
        }
    }

    pub fn family(&self) -> &str {
        match self {
            ControllerType::Smi(_) => "smi",
            ControllerType::Realtek(_, _) => "rtl",
            ControllerType::Phison(_) => "phison",
            ControllerType::Maxio(_) => "maxio",
            ControllerType::Marvell(_) => "marvell",
            ControllerType::Innogrit(_) => "innogrit",
            ControllerType::Tenafe(_) => "tenafe",
        }
    }
}

const SMI_FW_PREFIXES: &[(&str, &str)] = &[
    ("2260ROM:", "SM2260"),
    ("2262ROM:", "SM2262"),
    ("2262B0ROM:", "SM2262EN"),
    ("2262B1ROM:", "SM2262EN"),
    ("2262BCROM:", "SM2262EN"),
    ("2263ROM:", "SM2263EN"),
    ("2264ROM:", "SM2264"),
    ("2264ABROM:", "SM2264"),
    ("2265ABROM:", "SM2265"),
    ("2265", "SM2265"),
    ("2267ABROM:", "SM2267"),
    ("2267", "SM2267"),
    ("2268", "SM2268"),
    ("2269", "SM2269"),
    ("2270ROM:", "SM2270"),
    ("2270", "SM2270"),
    ("2508", "SM2508"),
    ("8366", "SM8366"),
];

const RTL_FW_PREFIXES: &[(&str, &str, RtlVariant)] = &[
    ("REALTEK_RL6447", "RTS5762/63", RtlVariant::V1),
    ("REALTEK_RL6577", "RTS5765/66", RtlVariant::V2),
    ("REALTEK_RL6817", "RTS5772", RtlVariant::V2),
];

const SMI_VID: u16 = 0x2646;

fn detect_realtek(fw: &str) -> Option<ControllerType> {
    for &(prefix, name, variant) in RTL_FW_PREFIXES {
        if fw.starts_with(prefix) {
            return Some(ControllerType::Realtek(name.to_string(), variant));
        }
    }
    None
}

fn detect_smi(fw: &str, model: &str, vid: u16, ssvid: u16) -> Option<ControllerType> {
    for &(prefix, name) in SMI_FW_PREFIXES {
        if fw.starts_with(prefix) {
            return Some(ControllerType::Smi(name.to_string()));
        }
    }
    if model.contains("SM22") || model.contains("SM25") || model.contains("SM83") {
        return Some(ControllerType::Smi("SMI (by model)".to_string()));
    }
    if vid == SMI_VID || ssvid == SMI_VID {
        return Some(ControllerType::Smi("SMI (by VID)".to_string()));
    }
    None
}

fn detect_tenafe(model: &str) -> Option<ControllerType> {
    if model == "Merak Nvme Ssd Controller" {
        return Some(ControllerType::Tenafe("Merak".to_string()));
    }
    None
}

fn probe_phison(dev: &NvmeDevice) -> Option<ControllerType> {
    let mut buf = [0u8; 4096];
    if dev
        .admin_read(0xD2, 0, 0, 0, 0, 0, 0, 0, &mut buf)
        .is_ok()
        && buf.windows(8).any(|w| w == b"PhIsOnNo")
    {
        return Some(ControllerType::Phison("Phison".to_string()));
    }
    None
}

fn probe_maxio(dev: &NvmeDevice) -> Option<ControllerType> {
    let mut buf = [0u8; 4096];
    for opcode in [0xC1, 0xC2] {
        if dev
            .admin_read(opcode, 0, 0, 0, 0, 0, 0, 0, &mut buf)
            .is_ok()
            && let Ok(s) = std::str::from_utf8(&buf)
            && s.contains(",MAP1")
        {
            return Some(ControllerType::Maxio("Maxio".to_string()));
        }
    }
    None
}

fn probe_marvell(dev: &NvmeDevice) -> Option<ControllerType> {
    let mut buf = [0u8; 4096];
    if dev
        .admin_read(0xFE, 0, 0, 0, 0, 0, 0, 0xA1, &mut buf)
        .is_ok()
    {
        if buf.starts_with(b"DM1160") {
            return Some(ControllerType::Marvell("DM1160".to_string()));
        }
        if buf.starts_with(b"DM1140") {
            return Some(ControllerType::Marvell("DM1140".to_string()));
        }
    }
    None
}

fn probe_innogrit(dev: &NvmeDevice) -> Option<ControllerType> {
    let mut buf = [0u8; 4096];
    if dev
        .admin_read(0xF2, 0, 0, 0, 0, 0, 0x54495247, 0x4F4E4E49, &mut buf)
        .is_ok()
        && buf.iter().any(|&b| b != 0)
    {
        let did_offset = 0x62E;
        let name = if did_offset + 2 <= buf.len() {
            let did = u16::from_le_bytes([buf[did_offset], buf[did_offset + 1]]);
            if did != 0 {
                format!("Innogrit (DID 0x{did:04X})")
            } else {
                "Innogrit".to_string()
            }
        } else {
            "Innogrit".to_string()
        };
        return Some(ControllerType::Innogrit(name));
    }
    None
}

pub fn detect(dev: &NvmeDevice, info: &crate::nvme::ControllerInfo) -> Option<ControllerType> {
    if let Some(ct) = detect_realtek(&info.firmware) {
        return Some(ct);
    }
    if let Some(ct) = detect_smi(&info.firmware, &info.model, info.vid, info.ssvid) {
        return Some(ct);
    }
    if let Some(ct) = detect_tenafe(&info.model) {
        return Some(ct);
    }
    if let Some(ct) = probe_phison(dev) {
        return Some(ct);
    }
    if let Some(ct) = probe_maxio(dev) {
        return Some(ct);
    }
    if let Some(ct) = probe_marvell(dev) {
        return Some(ct);
    }
    if let Some(ct) = probe_innogrit(dev) {
        return Some(ct);
    }
    None
}
