const MANUFACTURERS: &[(u8, &str)] = &[
    (0x01, "Spansion"),
    (0x04, "Fujitsu"),
    (0x07, "Renesas"),
    (0x20, "STMicro"),
    (0x2C, "Micron"),
    (0x45, "SanDisk"),
    (0x4A, "SMIC"),
    (0x51, "Qimonda"),
    (0x89, "Intel"),
    (0x92, "ESMT(PowerChip)"),
    (0x98, "Toshiba/Kioxia"),
    (0x9B, "YMTC"),
    (0xAD, "SK Hynix"),
    (0xB5, "Spectek"),
    (0xC2, "Macronix"),
    (0xC8, "ESMT(MIRA-PSC)"),
    (0xEC, "Samsung"),
    (0xEF, "Winbond"),
];

/// (description, offset, match_length, pattern_bytes)
/// Entries with offset=0 include the manufacturer byte in the pattern.
const MICRON_INTEL_SPECTEK_TABLE: &[(&str, u8, u8, &[u8])] = &[
    ("176L(B47R)", 1, 5, &[0xC3, 0x08, 0x32, 0xEA, 0x30]),
    ("176L(B47R)", 1, 5, &[0xD3, 0x89, 0x32, 0xEA, 0x30]),
    ("176L(B47R)", 1, 5, &[0xE3, 0x8A, 0x32, 0xEA, 0x30]),
    ("176L(B47T)", 1, 5, &[0xC3, 0x08, 0x32, 0xEA, 0x34]),
    ("176L(B47T)", 1, 5, &[0xD3, 0x89, 0x32, 0xEA, 0x34]),
    ("176L(B47T)", 1, 5, &[0xE3, 0x8A, 0x32, 0xEA, 0x34]),
    ("176L(N48R)", 1, 5, &[0xD3, 0x0C, 0x32, 0xEA, 0x30]),
    ("176L(N48R)", 1, 5, &[0xE3, 0x8D, 0x32, 0xEA, 0x30]),
    ("176L(N48R)", 1, 5, &[0xF3, 0x8E, 0x32, 0xEA, 0x30]),
    ("232L(B57T)", 1, 5, &[0xC3, 0x08, 0x32, 0xE6, 0x30]),
    ("232L(B57T)", 1, 5, &[0xD3, 0x89, 0x32, 0xE6, 0x30]),
    ("232L(B57T)", 1, 5, &[0xE3, 0x8A, 0x32, 0xE6, 0x30]),
    ("232L(B58R)", 1, 5, &[0xD3, 0x08, 0x32, 0xE8, 0x30]),
    ("232L(B58R)", 1, 5, &[0xE3, 0x89, 0x32, 0xE8, 0x30]),
    ("232L(B58R)", 1, 5, &[0xF3, 0x8A, 0x32, 0xE8, 0x30]),
    ("232L(B58R)", 1, 5, &[0xD3, 0x08, 0x32, 0xE8, 0x31]),
    ("232L(B58R)", 1, 5, &[0xE3, 0x89, 0x32, 0xE8, 0x31]),
    ("232L(B58R)", 1, 5, &[0xF3, 0x8A, 0x32, 0xE8, 0x31]),
    ("232L(N58R)", 1, 5, &[0xD3, 0x0C, 0x42, 0xEE, 0x30]),
    ("232L(N58R)", 1, 5, &[0xE3, 0x8D, 0x42, 0xEE, 0x30]),
    ("232L(N58R)", 1, 5, &[0xF3, 0x8E, 0x42, 0xEE, 0x30]),
    ("232L(N58R)", 1, 5, &[0xD3, 0x0C, 0x42, 0xEE, 0x31]),
    ("232L(N58R)", 1, 5, &[0xE3, 0x8D, 0x42, 0xEE, 0x31]),
    ("232L(N58R)", 1, 5, &[0xF3, 0x8E, 0x42, 0xEE, 0x31]),
    ("276L(B68S)", 1, 5, &[0xD3, 0x08, 0x32, 0xE8, 0x34]),
    ("276L(B68S)", 1, 5, &[0xE3, 0x89, 0x32, 0xE8, 0x34]),
    ("276L(B68S)", 1, 5, &[0xF3, 0x8A, 0x32, 0xE8, 0x34]),
    ("276L(B68S)", 1, 5, &[0xD3, 0x08, 0x32, 0xE8, 0x35]),
    ("276L(B68S)", 1, 5, &[0xE3, 0x89, 0x32, 0xE8, 0x35]),
    ("276L(B68S)", 1, 5, &[0xF3, 0x8A, 0x32, 0xE8, 0x35]),
    ("50nm(L52A)", 1, 4, &[0xD5, 0x94, 0x3E, 0x74]),
    ("50nm(L52A)", 1, 4, &[0xD7, 0xD5, 0x3E, 0x78]),
    ("34nm(M62A)", 1, 4, &[0x48, 0x00, 0x26, 0x89]),
    ("34nm(M62A)", 1, 4, &[0x68, 0x01, 0xA6, 0x89]),
    ("34nm(L63B)", 1, 4, &[0x68, 0x04, 0x46, 0x89]),
    ("34nm(L63B)", 1, 4, &[0x88, 0x05, 0xC6, 0x89]),
    ("34nm(L63B)", 1, 4, &[0x68, 0x04, 0x46, 0xA9]),
    ("25nm(M73A)", 1, 4, &[0x68, 0x00, 0x27, 0xA9]),
    ("25nm(M73A)", 1, 4, &[0x88, 0x01, 0xA7, 0xA9]),
    ("25nm(M73A)", 1, 4, &[0x68, 0x20, 0x27, 0xA9]),
    ("25nm(L73A)", 1, 4, &[0x68, 0x04, 0x4A, 0xA9]),
    ("25nm(L73A)", 1, 4, &[0x88, 0x05, 0xCA, 0xA9]),
    ("25nm(L74A)", 1, 5, &[0x88, 0x04, 0x4B, 0xA9, 0x00]),
    ("25nm(L74A)", 1, 5, &[0xA8, 0x05, 0xCB, 0xA9, 0x00]),
    ("25nm(L74A)", 1, 5, &[0x88, 0x24, 0x4B, 0xA9, 0x00]),
    ("20nm(L84A)", 1, 5, &[0x88, 0x24, 0x4B, 0xA9, 0x84]),
    ("20nm(L84A)", 1, 4, &[0x64, 0x44, 0x4B, 0xA9]),
    ("20nm(L84A)", 1, 4, &[0x84, 0xC5, 0x4B, 0xA9]),
    ("20nm(L84C)", 1, 4, &[0x64, 0x64, 0x3C, 0xA1]),
    ("20nm(L84C)", 1, 4, &[0x64, 0x64, 0x3C, 0xA5]),
    ("20nm(L84C)", 1, 4, &[0x84, 0xE5, 0x3C, 0xA5]),
    ("20nm(L85A)", 1, 4, &[0x84, 0x64, 0x3C, 0xA5]),
    ("20nm(L85A)", 1, 4, &[0xA4, 0xE5, 0x3C, 0xA5]),
    ("20nm(L85C)", 1, 4, &[0x84, 0x64, 0x3C, 0xA9]),
    ("20nm(L85C)", 1, 4, &[0xA4, 0xE5, 0x3C, 0xA9]),
    ("16nm(L95B)", 1, 4, &[0x84, 0x64, 0x54, 0xA9]),
    ("16nm(L95B)", 1, 4, &[0xA4, 0xE5, 0x54, 0xA9]),
    ("34nm(B63A)", 1, 4, &[0x68, 0x08, 0x56, 0x8A]),
    ("25nm(B74A)", 1, 4, &[0x88, 0x08, 0x5F, 0xA9]),
    ("25nm(B74A)", 1, 4, &[0x88, 0x08, 0x5F, 0x89]),
    ("25nm(B74A)", 1, 4, &[0x88, 0x28, 0x5F, 0xA9]),
    ("25nm(B74A)", 1, 4, &[0xA8, 0x09, 0xDF, 0x89]),
    ("25nm(B74A)", 1, 4, &[0xA8, 0x09, 0xDF, 0xA9]),
    ("20nm(B85T)", 1, 4, &[0x84, 0x78, 0x63, 0xA9]),
    ("20nm(B85T)", 1, 4, &[0x84, 0x78, 0x7B, 0xA9]),
    ("20nm(B85T)", 1, 4, &[0xA4, 0xF9, 0x63, 0xA9]),
    ("20nm(B85T)", 1, 4, &[0xA4, 0xF9, 0x7B, 0xA9]),
    ("16nm(B95A)", 1, 4, &[0x84, 0x48, 0x63, 0xA9]),
    ("32L(L04A)", 1, 4, &[0x64, 0x44, 0x32, 0xA5]),
    ("32L(L05B)", 1, 4, &[0x84, 0x44, 0x34, 0xAA]),
    ("32L(B05A)", 1, 4, &[0x84, 0x58, 0x32, 0xA1]),
    ("32L(L05A)", 1, 4, &[0xA4, 0x64, 0x34, 0xAA]),
    ("32L(L06A)", 1, 4, &[0xA4, 0xE4, 0x34, 0x8A]),
    ("32L(L06B)", 1, 4, &[0xA4, 0x64, 0x32, 0xAA]),
    ("32L(L06B)", 1, 4, &[0xC4, 0xE5, 0x32, 0xAA]),
    ("32L(B0KB)", 1, 4, &[0xB4, 0x78, 0x32, 0xAA]),
    ("32L(B0KB)", 1, 4, &[0xCC, 0xF9, 0x32, 0xAA]),
    ("64L(B16A)", 1, 4, &[0xA4, 0x08, 0x32, 0xA1]),
    ("64L(B16A)", 1, 4, &[0xA4, 0x88, 0x32, 0xA1]),
    ("64L(B16A)", 1, 4, &[0xC4, 0x89, 0x32, 0xA1]),
    ("64L(B17A)", 1, 4, &[0xC4, 0x08, 0x32, 0xA6]),
    ("64L(B17A)", 1, 4, &[0xD4, 0x89, 0x32, 0xA6]),
    ("64L(B17A)", 1, 4, &[0xE4, 0x8A, 0x32, 0xA6]),
    ("64L(N18A)", 1, 4, &[0xD4, 0x0C, 0x32, 0xAA]),
    ("96L(B27A)", 1, 4, &[0xC4, 0x18, 0x32, 0xA2]),
    ("96L(B27A)", 1, 4, &[0xD4, 0x99, 0x32, 0xA2]),
    ("96L(B27A)", 1, 4, &[0xE4, 0x9A, 0x32, 0xA2]),
    ("96L(B27B)", 1, 5, &[0xC3, 0x08, 0x32, 0xE6, 0x00]),
    ("96L(B27B)", 1, 4, &[0xD3, 0x89, 0x32, 0xE6]),
    ("96L(B27B)", 1, 4, &[0xE3, 0x8A, 0x32, 0xE6]),
    ("96L(N28A)", 1, 4, &[0xD3, 0x1C, 0x32, 0xC6]),
    ("96L(N28A)", 1, 4, &[0xD3, 0x9C, 0x32, 0xC6]),
    ("96L(N28A)", 1, 4, &[0xE3, 0x9D, 0x32, 0xC6]),
    ("96L(N28A)", 1, 4, &[0xF3, 0x9E, 0x32, 0xC6]),
    ("96L(M26A)", 1, 4, &[0xA3, 0x60, 0x32, 0xC6]),
    ("128L(B36R)", 1, 4, &[0xA3, 0x78, 0x32, 0xE5]),
    ("128L(B37R)", 1, 4, &[0xC3, 0x78, 0x32, 0xEA]),
    ("144L(N38A)", 0, 5, &[0x89, 0xD3, 0xAC, 0x32, 0xC6]),
    ("144L(N38A)", 0, 5, &[0x89, 0xE3, 0xAD, 0x32, 0xC6]),
    ("144L(N38B)", 0, 5, &[0x89, 0xD3, 0xAC, 0x32, 0xC2]),
    ("144L(N38B)", 0, 5, &[0x89, 0xE3, 0xAD, 0x32, 0xC2]),
    ("192L(Q5171A)", 0, 5, &[0x89, 0x09, 0x28, 0x32, 0xC2]),
    ("192L(Q5171A)", 0, 5, &[0x89, 0x09, 0x29, 0x32, 0xC2]),
    ("192L(Q5171A)", 0, 5, &[0x89, 0x09, 0x2A, 0x32, 0xC2]),
    ("192L(Q5171A)", 0, 5, &[0x89, 0x09, 0x2B, 0x32, 0xC2]),
    ("192L(N4PA)", 0, 5, &[0x89, 0x05, 0x04, 0x32, 0xC2]),
    ("192L(N4PA)", 0, 5, &[0x89, 0x05, 0x05, 0x32, 0xC2]),
    ("192L(N4PA)", 0, 5, &[0x89, 0x05, 0x06, 0x32, 0xC2]),
    ("192L(N4PA)", 0, 5, &[0x89, 0x05, 0x07, 0x32, 0xC2]),
];

const YMTC_TABLE: &[(&str, u8, u8, &[u8])] = &[
    ("(x1-9050)", 1, 4, &[0xC3, 0x48, 0x25, 0x10]),
    ("(x2-6070)", 1, 4, &[0xD5, 0x58, 0x8D, 0x20]),
    ("(x2-9060)", 1, 4, &[0xC4, 0x28, 0x49, 0x20]),
    ("(x2-9060)", 1, 4, &[0xC5, 0x29, 0x49, 0x20]),
    ("-128L(x3-9060)", 1, 4, &[0xC4, 0x28, 0x49, 0x30]),
    ("-128L(x3-9060)", 1, 4, &[0xC5, 0x29, 0x49, 0x30]),
    ("-232L(x3-9070)", 1, 4, &[0xC5, 0x58, 0x71, 0x30]),
    ("-232L(x3-9070)", 1, 4, &[0xC6, 0x59, 0x71, 0x30]),
    ("-232L(x3-6070)", 1, 4, &[0xC5, 0x5C, 0x55, 0x30]),
];

const TOSHIBA_SANDISK_TECH: &[(u8, &str)] = &[
    (0x00, "A19nm"),
    (0x01, "15nm"),
    (0x02, "70nm"),
    (0x03, "56nm"),
    (0x04, "43nm"),
    (0x05, "32nm"),
    (0x06, "24nm"),
    (0x07, "19nm"),
    (0x20, "48L BiCS2"),
    (0x21, "64L BiCS3"),
    (0x22, "96L BiCS4"),
    (0x23, "112L BiCS5"),
    (0x24, "162L BiCS6"),
    (0x25, "218L BiCS8"),
];

/// Combined SK Hynix table: RTL values (raw byte[5]) plus SMI-only values.
/// RTL values are the actual NAND ID byte[5]; SMI values that don't overlap are appended.
const HYNIX_TECH: &[(u8, &str)] = &[
    // RTL table (raw byte[5])
    (0x25, "41nm"),
    (0x26, "32nm"),
    (0x27, "26nm"),
    (0x28, "26nm"),
    (0x29, "20nm"),
    (0x2A, "20nm"),
    (0x36, "16nm"),
    (0x37, "16nm"),
    (0x3A, "16nm"),
    (0x3B, "16nm"),
    (0x3C, "16nm"),
    (0x3D, "16nm"),
    (0x40, "16nm"),
    (0x44, "16nm"),
    (0x48, "14nm"),
    (0x49, "3dv1"),
    (0x4A, "14nm"),
    (0x4B, "3dv2-36L"),
    (0x50, "3dv3-48L"),
    (0x55, "3dv4-72L"),
    (0x5A, "3dv5-96L"),
    (0x5B, "3dv5-96L"),
    (0x60, "3dv6-128L"),
    (0x61, "3dv6-128L"),
    (0x65, "3dv7-176L"),
    (0x66, "3dv7-176L"),
    (0x76, "3dv8-238L"),
    // SMI-only values not present in RTL table
    (0x56, "16nm"),
    (0x57, "16nm"),
    (0x58, "14nm"),
    (0x64, "16nm"),
    (0x67, "16nm"),
    (0x68, "16nm"),
    (0x69, "3dv1"),
    (0x6A, "16nm"),
    (0x6B, "3dv2-36L"),
    (0x6C, "3dv3-48L"),
    (0x6D, "3dv4-72L"),
    (0x6E, "3dv5-96L"),
    (0x6F, "3dv5-96L"),
    (0x70, "3dv6-128L"),
    (0x71, "3dv6-128L"),
    (0x72, "3dv7-176L"),
    (0x73, "3dv7-176L"),
    (0x7A, "3dv8-238L"),
    (0x7B, "14nm"),
    (0x7C, "26nm"),
    (0x7D, "20nm"),
];

/// Samsung RTL table uses byte[5] & 0x7F as key.
const SAMSUNG_TECH_RTL: &[(u8, &str)] = &[
    (0x40, "51nm"),
    (0x41, "42nm"),
    (0x42, "32nm"),
    (0x43, "27nm"),
    (0x44, "21nm"),
    (0x45, "19nm"),
    (0x46, "16nm"),
    (0x47, "3dv1-24L"),
    (0x48, "3dv2-32L"),
    (0x49, "3dv3-48L"),
    (0x4A, "14nm"),
    (0x4B, "3dv4-64L"),
    (0x4C, "3dv5-92L"),
    (0x4D, "3dv6-136L"),
    (0x4E, "3dv7-176L"),
    (0x4F, "3dv8-236L"),
];

/// Samsung SMI-only entries that use raw byte[5] (no mask).
const SAMSUNG_TECH_SMI_EXTRA: &[(u8, &str)] = &[
    (0x86, "3dv6e-136L"),
    (0x87, "3dv7-176L"),
];

pub fn manufacturer_name(id: u8) -> &'static str {
    for &(mid, name) in MANUFACTURERS {
        if mid == id {
            return name;
        }
    }
    "Unknown"
}

fn lookup_table<'a>(table: &[(u8, &'a str)], key: u8) -> Option<&'a str> {
    for &(k, v) in table {
        if k == key {
            return Some(v);
        }
    }
    None
}

fn cell_type(fid: &[u8]) -> &'static str {
    let cell_bits = (fid[2] >> 2) & 3;
    if matches!(fid[0], 0x2C | 0x89 | 0xB5) && matches!(fid[1], 0x05 | 0x09) {
        return match cell_bits {
            1 => "TLC",
            2 => "QLC",
            3 => "PLC",
            _ => "",
        };
    }
    if fid[0] == 0x9B
        && fid.len() >= 5
        && fid[1] == 0xD5
        && fid[2] == 0x58
        && fid[3] == 0x8D
        && fid[4] == 0x20
    {
        return "QLC";
    }
    match cell_bits {
        0 => "SLC",
        1 => "MLC",
        2 => "TLC",
        3 => "QLC",
        _ => "",
    }
}

fn page_size(fid: &[u8]) -> &'static str {
    match fid[0] {
        0x98 | 0x45 | 0xEC | 0xAD => match fid[3] & 3 {
            0 => "2k",
            1 => "4k",
            2 => "8k",
            3 => "16k",
            _ => "",
        },
        0x9B => match fid[3] & 1 {
            0 => "8k",
            1 => "16k",
            _ => "",
        },
        _ => "",
    }
}

fn identify_micron_intel_spectek(fid: &[u8]) -> Option<&'static str> {
    for &(desc, offset, length, pattern) in MICRON_INTEL_SPECTEK_TABLE {
        let off = offset as usize;
        let len = length as usize;
        if off + len <= fid.len() && fid[off..off + len] == *pattern {
            return Some(desc);
        }
    }
    None
}

fn identify_ymtc(fid: &[u8]) -> Option<String> {
    let mut table_desc: Option<&str> = None;
    for &(desc, offset, length, pattern) in YMTC_TABLE {
        let off = offset as usize;
        let len = length as usize;
        if off + len <= fid.len() && fid[off..off + len] == *pattern {
            table_desc = Some(desc);
            break;
        }
    }

    let gen_bits = if fid.len() > 4 {
        (fid[4] >> 4) & 7
    } else {
        0
    };
    let gen_str = match gen_bits {
        1 => "3dv2-64L",
        2 => "3dv3-128L",
        3 => "3dv4",
        4 => "3dv5",
        _ => "",
    };

    match (table_desc, gen_str) {
        (Some(td), gs) if !gs.is_empty() => Some(format!("{gs}{td}")),
        (Some(td), _) => Some(td.to_string()),
        (None, gs) if !gs.is_empty() => Some(gs.to_string()),
        _ => None,
    }
}

fn identify_toshiba_sandisk(fid: &[u8]) -> Option<&'static str> {
    if fid.len() < 6 {
        return None;
    }
    lookup_table(TOSHIBA_SANDISK_TECH, fid[5] & 0x27)
}

fn identify_hynix(fid: &[u8]) -> Option<&'static str> {
    if fid.len() < 6 {
        return None;
    }
    lookup_table(HYNIX_TECH, fid[5])
}

fn identify_samsung(fid: &[u8]) -> Option<&'static str> {
    if fid.len() < 6 {
        return None;
    }
    let raw = fid[5];
    if let Some(v) = lookup_table(SAMSUNG_TECH_SMI_EXTRA, raw) {
        return Some(v);
    }
    lookup_table(SAMSUNG_TECH_RTL, raw & 0x7F)
}

pub fn describe_flash(flash_id: &[u8]) -> String {
    if flash_id.len() < 6 {
        return "Unknown".to_string();
    }
    let mfr_id = flash_id[0];
    let mfr_name = manufacturer_name(mfr_id);

    let tech: Option<String> = match mfr_id {
        0x2C | 0x89 | 0xB5 => identify_micron_intel_spectek(flash_id).map(String::from),
        0x9B => identify_ymtc(flash_id),
        0x98 | 0x45 => identify_toshiba_sandisk(flash_id).map(String::from),
        0xAD => identify_hynix(flash_id).map(String::from),
        0xEC => identify_samsung(flash_id).map(String::from),
        _ => None,
    };

    let cell = cell_type(flash_id);
    let page = page_size(flash_id);

    let mut parts = vec![mfr_name.to_string()];
    if let Some(t) = tech {
        parts.push(t);
    }
    if !cell.is_empty() {
        parts.push(cell.to_string());
    }
    if !page.is_empty() {
        parts.push(page.to_string());
    }
    parts.join(" ")
}

pub fn format_flash_id_hex(flash_id: &[u8]) -> String {
    flash_id
        .iter()
        .map(|b| format!("0x{b:02x}"))
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manufacturer_names() {
        assert_eq!(manufacturer_name(0x2C), "Micron");
        assert_eq!(manufacturer_name(0x89), "Intel");
        assert_eq!(manufacturer_name(0xEC), "Samsung");
        assert_eq!(manufacturer_name(0xFF), "Unknown");
    }

    #[test]
    fn test_format_hex() {
        let id = [0x89, 0xD3, 0xAC, 0x32, 0xC6, 0x00, 0x00, 0x00];
        assert_eq!(
            format_flash_id_hex(&id),
            "0x89,0xd3,0xac,0x32,0xc6,0x00,0x00,0x00"
        );
    }

    #[test]
    fn test_intel_144l() {
        let id = [0x89, 0xD3, 0xAC, 0x32, 0xC6, 0x00, 0x00, 0x00];
        let desc = describe_flash(&id);
        assert!(desc.contains("Intel"));
        assert!(desc.contains("144L(N38A)"));
        assert!(desc.contains("QLC"));
    }

    #[test]
    fn test_micron_176l() {
        let id = [0x2C, 0xC3, 0x08, 0x32, 0xEA, 0x30, 0x00, 0x00];
        let desc = describe_flash(&id);
        assert!(desc.contains("Micron"));
        assert!(desc.contains("176L(B47R)"));
    }

    #[test]
    fn test_samsung_rtl() {
        let id = [0xEC, 0xA1, 0x08, 0x02, 0x00, 0x4E, 0x00, 0x00];
        let desc = describe_flash(&id);
        assert!(desc.contains("Samsung"));
        assert!(desc.contains("3dv7-176L"));
        assert!(desc.contains("8k"));
    }

    #[test]
    fn test_samsung_smi_extra() {
        let id = [0xEC, 0xA1, 0x08, 0x02, 0x00, 0x86, 0x00, 0x00];
        let desc = describe_flash(&id);
        assert!(desc.contains("Samsung"));
        assert!(desc.contains("3dv6e-136L"));
    }

    #[test]
    fn test_toshiba_bics5() {
        let id = [0x98, 0xA1, 0x08, 0x02, 0x00, 0x23, 0x00, 0x00];
        let desc = describe_flash(&id);
        assert!(desc.contains("Toshiba/Kioxia"));
        assert!(desc.contains("112L BiCS5"));
        assert!(desc.contains("8k"));
    }

    #[test]
    fn test_hynix_3dv7() {
        let id = [0xAD, 0xA1, 0x08, 0x02, 0x00, 0x65, 0x00, 0x00];
        let desc = describe_flash(&id);
        assert!(desc.contains("SK Hynix"));
        assert!(desc.contains("3dv7-176L"));
    }

    #[test]
    fn test_ymtc_qlc_special() {
        let id = [0x9B, 0xD5, 0x58, 0x8D, 0x20, 0x00, 0x00, 0x00];
        let desc = describe_flash(&id);
        assert!(desc.contains("YMTC"));
        assert!(desc.contains("QLC"));
    }

    #[test]
    fn test_intel_special_cell_type() {
        let id = [0x89, 0x09, 0x28, 0x32, 0xC2, 0x00, 0x00, 0x00];
        let desc = describe_flash(&id);
        assert!(desc.contains("Intel"));
        assert!(desc.contains("192L(Q5171A)"));
        assert!(desc.contains("QLC"));
    }

    #[test]
    fn test_cell_type_general() {
        assert_eq!(cell_type(&[0xEC, 0x00, 0x00, 0x00, 0x00, 0x00]), "SLC");
        assert_eq!(cell_type(&[0xEC, 0x00, 0x04, 0x00, 0x00, 0x00]), "MLC");
        assert_eq!(cell_type(&[0xEC, 0x00, 0x08, 0x00, 0x00, 0x00]), "TLC");
        assert_eq!(cell_type(&[0xEC, 0x00, 0x0C, 0x00, 0x00, 0x00]), "QLC");
    }

    #[test]
    fn test_short_id() {
        assert_eq!(describe_flash(&[0x89, 0x00, 0x00]), "Unknown");
    }
}
