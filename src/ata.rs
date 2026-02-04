use std::ffi::CString;

const SG_IO: u64 = 0x2285;
const SG_DXFER_NONE: i32 = -1;
const SG_DXFER_TO_DEV: i32 = -2;
const SG_DXFER_FROM_DEV: i32 = -3;
const ATA_PT16_OPCODE: u8 = 0x85;
const ATA_PT16_CDB_LEN: u8 = 16;
const SENSE_BUF_LEN: u8 = 32;
const TIMEOUT_MS: u32 = 10_000;

const PROTO_NON_DATA: u8 = 3;
const PROTO_PIO_DATA_IN: u8 = 4;
const PROTO_PIO_DATA_OUT: u8 = 5;
const PROTO_DMA: u8 = 6;

// CDB byte 2 values: chk_cond=1(bit5), t_dir(bit3), byte_block=1(bit2), t_length=2(bits1:0)
const CDB2_READ: u8 = 0x2E;
const CDB2_WRITE: u8 = 0x26;
const CDB2_NON_DATA: u8 = 0x20;

#[repr(C)]
struct SgIoHdr {
    interface_id: i32,
    dxfer_direction: i32,
    cmd_len: u8,
    mx_sb_len: u8,
    iovec_count: u16,
    dxfer_len: u32,
    dxferp: *mut u8,
    cmdp: *const u8,
    sbp: *mut u8,
    timeout: u32,
    flags: u32,
    pack_id: i32,
    usr_ptr: *mut u8,
    status: u8,
    masked_status: u8,
    msg_status: u8,
    sb_len_wr: u8,
    host_status: u16,
    driver_status: u16,
    resid: i32,
    duration: u32,
    info: u32,
}

impl SgIoHdr {
    fn zeroed() -> Self {
        SgIoHdr {
            interface_id: 0,
            dxfer_direction: 0,
            cmd_len: 0,
            mx_sb_len: 0,
            iovec_count: 0,
            dxfer_len: 0,
            dxferp: std::ptr::null_mut(),
            cmdp: std::ptr::null(),
            sbp: std::ptr::null_mut(),
            timeout: 0,
            flags: 0,
            pack_id: 0,
            usr_ptr: std::ptr::null_mut(),
            status: 0,
            masked_status: 0,
            msg_status: 0,
            sb_len_wr: 0,
            host_status: 0,
            driver_status: 0,
            resid: 0,
            duration: 0,
            info: 0,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub struct AtaDevice {
    fd: i32,
}

#[allow(clippy::too_many_arguments)]
impl AtaDevice {
    pub fn open(path: &str) -> Result<Self, String> {
        let c_path =
            CString::new(path).map_err(|e| format!("invalid device path '{}': {}", path, e))?;
        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDWR) };
        if fd < 0 {
            let errno = unsafe { *libc::__errno_location() };
            return Err(format!(
                "failed to open '{}': {} (errno {})",
                path,
                errno_to_str(errno),
                errno
            ));
        }
        Ok(AtaDevice { fd })
    }

    pub fn ata_identify(&self) -> Result<[u8; 512], String> {
        let mut buf = [0u8; 512];
        self.ata_read(0xEC, 0, 1, 0, 0, 0, 0xE0, &mut buf)?;
        Ok(buf)
    }

    pub fn ata_read(
        &self,
        command: u8,
        features: u8,
        count: u8,
        lba_low: u8,
        lba_mid: u8,
        lba_high: u8,
        device: u8,
        buf: &mut [u8],
    ) -> Result<(), String> {
        let cdb = build_cdb(
            PROTO_PIO_DATA_IN,
            CDB2_READ,
            command,
            features,
            count,
            lba_low,
            lba_mid,
            lba_high,
            device,
        );
        self.sg_io(&cdb, SG_DXFER_FROM_DEV, buf.as_mut_ptr(), buf.len(), command)
    }

    pub fn ata_dma_read(
        &self,
        command: u8,
        features: u8,
        count: u8,
        lba_low: u8,
        lba_mid: u8,
        lba_high: u8,
        device: u8,
        buf: &mut [u8],
    ) -> Result<(), String> {
        let cdb = build_cdb(
            PROTO_DMA,
            CDB2_READ,
            command,
            features,
            count,
            lba_low,
            lba_mid,
            lba_high,
            device,
        );
        self.sg_io(&cdb, SG_DXFER_FROM_DEV, buf.as_mut_ptr(), buf.len(), command)
    }

    pub fn ata_write(
        &self,
        command: u8,
        features: u8,
        count: u8,
        lba_low: u8,
        lba_mid: u8,
        lba_high: u8,
        device: u8,
        buf: &[u8],
    ) -> Result<(), String> {
        let cdb = build_cdb(
            PROTO_PIO_DATA_OUT,
            CDB2_WRITE,
            command,
            features,
            count,
            lba_low,
            lba_mid,
            lba_high,
            device,
        );
        self.sg_io(&cdb, SG_DXFER_TO_DEV, buf.as_ptr() as *mut u8, buf.len(), command)
    }

    pub fn ata_no_data(
        &self,
        command: u8,
        features: u8,
        count: u8,
        lba_low: u8,
        lba_mid: u8,
        lba_high: u8,
        device: u8,
    ) -> Result<(), String> {
        let cdb = build_cdb(
            PROTO_NON_DATA,
            CDB2_NON_DATA,
            command,
            features,
            count,
            lba_low,
            lba_mid,
            lba_high,
            device,
        );
        self.sg_io(&cdb, SG_DXFER_NONE, std::ptr::null_mut(), 0, command)
    }

    pub fn ata_read_ext(
        &self,
        command: u8,
        features: u8,
        count: u8,
        lba_low: u8,
        lba_mid: u8,
        lba_high: u8,
        device: u8,
        prev_features: u8,
        prev_count: u8,
        prev_lba_low: u8,
        prev_lba_mid: u8,
        prev_lba_high: u8,
        buf: &mut [u8],
    ) -> Result<(), String> {
        let cdb = build_cdb_ext(
            PROTO_PIO_DATA_IN,
            CDB2_READ,
            command,
            features,
            count,
            lba_low,
            lba_mid,
            lba_high,
            device,
            prev_features,
            prev_count,
            prev_lba_low,
            prev_lba_mid,
            prev_lba_high,
        );
        self.sg_io(&cdb, SG_DXFER_FROM_DEV, buf.as_mut_ptr(), buf.len(), command)
    }

    pub fn ata_no_data_ext(
        &self,
        command: u8,
        features: u8,
        count: u8,
        lba_low: u8,
        lba_mid: u8,
        lba_high: u8,
        device: u8,
        prev_features: u8,
        prev_count: u8,
        prev_lba_low: u8,
        prev_lba_mid: u8,
        prev_lba_high: u8,
    ) -> Result<(), String> {
        let cdb = build_cdb_ext(
            PROTO_NON_DATA,
            CDB2_NON_DATA,
            command,
            features,
            count,
            lba_low,
            lba_mid,
            lba_high,
            device,
            prev_features,
            prev_count,
            prev_lba_low,
            prev_lba_mid,
            prev_lba_high,
        );
        self.sg_io(&cdb, SG_DXFER_NONE, std::ptr::null_mut(), 0, command)
    }

    fn sg_io(
        &self,
        cdb: &[u8; 16],
        direction: i32,
        dxferp: *mut u8,
        dxfer_len: usize,
        command: u8,
    ) -> Result<(), String> {
        let mut sense = [0u8; SENSE_BUF_LEN as usize];
        let mut hdr = SgIoHdr::zeroed();
        hdr.interface_id = b'S' as i32;
        hdr.dxfer_direction = direction;
        hdr.cmd_len = ATA_PT16_CDB_LEN;
        hdr.mx_sb_len = SENSE_BUF_LEN;
        hdr.dxfer_len = dxfer_len as u32;
        hdr.dxferp = dxferp;
        hdr.cmdp = cdb.as_ptr();
        hdr.sbp = sense.as_mut_ptr();
        hdr.timeout = TIMEOUT_MS;

        let ret = unsafe { libc::ioctl(self.fd, SG_IO, &mut hdr as *mut SgIoHdr) };
        if ret < 0 {
            let errno = unsafe { *libc::__errno_location() };
            return Err(format!(
                "sg_io ioctl failed: {} (errno {}, command 0x{:02x})",
                errno_to_str(errno),
                errno,
                command
            ));
        }

        // DRIVER_SENSE (0x08) is expected when CK_COND=1 is set in CDB byte 2
        if hdr.host_status != 0 || (hdr.driver_status & !0x08) != 0 {
            return Err(format!(
                "sg_io transport error: host_status=0x{:04x}, driver_status=0x{:04x}, command 0x{:02x}",
                hdr.host_status, hdr.driver_status, command
            ));
        }

        // Parse ATA Status Return descriptor (type 0x09) from descriptor format sense data.
        // Descriptor layout at sense[8]: [type=0x09][len=0x0C][extend][error]...[device][status]
        if hdr.sb_len_wr >= 22 && sense[0] == 0x72 && sense[8] == 0x09 && sense[9] == 0x0C {
            let ata_status = sense[21];
            let ata_error = sense[11];
            if ata_status & 0x01 != 0 {
                return Err(format!(
                    "ata command 0x{:02x} failed: status=0x{:02x}, error=0x{:02x}",
                    command, ata_status, ata_error
                ));
            }
        }

        Ok(())
    }
}

impl Drop for AtaDevice {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn build_cdb(
    protocol: u8,
    cdb2: u8,
    command: u8,
    features: u8,
    count: u8,
    lba_low: u8,
    lba_mid: u8,
    lba_high: u8,
    device: u8,
) -> [u8; 16] {
    [
        ATA_PT16_OPCODE,
        protocol << 1,
        cdb2,
        0,
        features,
        0,
        count,
        0,
        lba_low,
        0,
        lba_mid,
        0,
        lba_high,
        device,
        command,
        0,
    ]
}

#[allow(clippy::too_many_arguments)]
fn build_cdb_ext(
    protocol: u8,
    cdb2: u8,
    command: u8,
    features: u8,
    count: u8,
    lba_low: u8,
    lba_mid: u8,
    lba_high: u8,
    device: u8,
    prev_features: u8,
    prev_count: u8,
    prev_lba_low: u8,
    prev_lba_mid: u8,
    prev_lba_high: u8,
) -> [u8; 16] {
    [
        ATA_PT16_OPCODE,
        (protocol << 1) | 1, // extend=1 for 48-bit LBA
        cdb2,
        prev_features,
        features,
        prev_count,
        count,
        prev_lba_low,
        lba_low,
        prev_lba_mid,
        lba_mid,
        prev_lba_high,
        lba_high,
        device,
        command,
        0,
    ]
}

pub struct AtaIdentify {
    pub model: String,
    pub serial: String,
    pub firmware: String,
}

pub fn parse_ata_identify(data: &[u8; 512]) -> AtaIdentify {
    let serial = ata_string_trim(&data[20..40]);
    let firmware = ata_string_trim(&data[46..54]);
    let model = ata_string_trim(&data[54..94]);
    AtaIdentify {
        model,
        serial,
        firmware,
    }
}

/// ATA strings store each 16-bit word with the high byte first (byte-swapped relative to host),
/// so adjacent bytes within each pair must be swapped to produce the correct ASCII.
fn ata_string_trim(raw: &[u8]) -> String {
    let mut out = Vec::with_capacity(raw.len());
    for pair in raw.chunks_exact(2) {
        out.push(pair[1]);
        out.push(pair[0]);
    }
    let s: String = out
        .iter()
        .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { ' ' })
        .collect();
    s.trim().to_string()
}

fn errno_to_str(errno: i32) -> &'static str {
    match errno {
        libc::EACCES => "permission denied",
        libc::ENOENT => "no such file or directory",
        libc::EBUSY => "device busy",
        libc::EIO => "i/o error",
        libc::EINVAL => "invalid argument",
        libc::ENOTTY => "inappropriate ioctl for device",
        libc::ENOMEM => "out of memory",
        libc::EPERM => "operation not permitted",
        libc::ENODEV => "no such device",
        _ => "unknown error",
    }
}
