use std::ffi::CString;

const NVME_IOCTL_ADMIN_CMD: u64 = 0xC0484E41;
const TIMEOUT_MS: u32 = 10_000;

#[repr(C)]
pub struct NvmeAdminCmd {
    pub opcode: u8,
    pub flags: u8,
    pub rsvd1: u16,
    pub nsid: u32,
    pub cdw2: u32,
    pub cdw3: u32,
    pub metadata: u64,
    pub addr: u64,
    pub metadata_len: u32,
    pub data_len: u32,
    pub cdw10: u32,
    pub cdw11: u32,
    pub cdw12: u32,
    pub cdw13: u32,
    pub cdw14: u32,
    pub cdw15: u32,
    pub timeout_ms: u32,
    pub result: u32,
}

impl NvmeAdminCmd {
    fn zeroed() -> Self {
        NvmeAdminCmd {
            opcode: 0,
            flags: 0,
            rsvd1: 0,
            nsid: 0,
            cdw2: 0,
            cdw3: 0,
            metadata: 0,
            addr: 0,
            metadata_len: 0,
            data_len: 0,
            cdw10: 0,
            cdw11: 0,
            cdw12: 0,
            cdw13: 0,
            cdw14: 0,
            cdw15: 0,
            timeout_ms: 0,
            result: 0,
        }
    }
}

pub struct NvmeDevice {
    fd: i32,
}

impl NvmeDevice {
    pub fn open(path: &str) -> Result<Self, String> {
        let c_path =
            CString::new(path).map_err(|e| format!("invalid device path '{}': {}", path, e))?;
        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDONLY) };
        if fd < 0 {
            let errno = unsafe { *libc::__errno_location() };
            return Err(format!(
                "failed to open '{}': {} (errno {})",
                path,
                errno_to_str(errno),
                errno
            ));
        }
        Ok(NvmeDevice { fd })
    }

    pub fn admin_read(
        &self,
        opcode: u8,
        nsid: u32,
        cdw10: u32,
        cdw11: u32,
        cdw12: u32,
        cdw13: u32,
        cdw14: u32,
        cdw15: u32,
        buf: &mut [u8],
    ) -> Result<u32, String> {
        let mut cmd = NvmeAdminCmd::zeroed();
        cmd.opcode = opcode;
        cmd.nsid = nsid;
        cmd.cdw10 = cdw10;
        cmd.cdw11 = cdw11;
        cmd.cdw12 = cdw12;
        cmd.cdw13 = cdw13;
        cmd.cdw14 = cdw14;
        cmd.cdw15 = cdw15;
        cmd.addr = buf.as_mut_ptr() as u64;
        cmd.data_len = buf.len() as u32;
        cmd.timeout_ms = TIMEOUT_MS;

        self.submit_admin_cmd(&mut cmd)
    }

    pub fn admin_write(
        &self,
        opcode: u8,
        nsid: u32,
        cdw10: u32,
        cdw11: u32,
        cdw12: u32,
        cdw13: u32,
        cdw14: u32,
        cdw15: u32,
        buf: &[u8],
    ) -> Result<u32, String> {
        let mut cmd = NvmeAdminCmd::zeroed();
        cmd.opcode = opcode;
        cmd.nsid = nsid;
        cmd.cdw10 = cdw10;
        cmd.cdw11 = cdw11;
        cmd.cdw12 = cdw12;
        cmd.cdw13 = cdw13;
        cmd.cdw14 = cdw14;
        cmd.cdw15 = cdw15;
        cmd.addr = buf.as_ptr() as u64;
        cmd.data_len = buf.len() as u32;
        cmd.timeout_ms = TIMEOUT_MS;

        self.submit_admin_cmd(&mut cmd)
    }

    pub fn admin_no_data(
        &self,
        opcode: u8,
        nsid: u32,
        cdw10: u32,
        cdw11: u32,
        cdw12: u32,
        cdw13: u32,
        cdw14: u32,
        cdw15: u32,
    ) -> Result<u32, String> {
        let mut cmd = NvmeAdminCmd::zeroed();
        cmd.opcode = opcode;
        cmd.nsid = nsid;
        cmd.cdw10 = cdw10;
        cmd.cdw11 = cdw11;
        cmd.cdw12 = cdw12;
        cmd.cdw13 = cdw13;
        cmd.cdw14 = cdw14;
        cmd.cdw15 = cdw15;
        cmd.timeout_ms = TIMEOUT_MS;

        self.submit_admin_cmd(&mut cmd)
    }

    /// Opcode 0x06, CNS=1 (cdw10=1)
    pub fn identify_controller(&self) -> Result<[u8; 4096], String> {
        let mut buf = [0u8; 4096];
        self.admin_read(0x06, 0, 1, 0, 0, 0, 0, 0, &mut buf)?;
        Ok(buf)
    }

    fn submit_admin_cmd(&self, cmd: &mut NvmeAdminCmd) -> Result<u32, String> {
        let ret = unsafe { libc::ioctl(self.fd, NVME_IOCTL_ADMIN_CMD, cmd as *mut NvmeAdminCmd) };
        if ret < 0 {
            let errno = unsafe { *libc::__errno_location() };
            return Err(format!(
                "nvme ioctl failed: {} (errno {}, opcode 0x{:02x})",
                errno_to_str(errno),
                errno,
                cmd.opcode
            ));
        }
        Ok(cmd.result)
    }
}

impl Drop for NvmeDevice {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}

pub struct ControllerInfo {
    pub vid: u16,
    pub ssvid: u16,
    pub serial: String,
    pub model: String,
    pub firmware: String,
}

pub fn parse_identify(data: &[u8; 4096]) -> ControllerInfo {
    let vid = u16::from_le_bytes([data[0], data[1]]);
    let ssvid = u16::from_le_bytes([data[2], data[3]]);
    let serial = ascii_trim(&data[4..24]);
    let model = ascii_trim(&data[24..64]);
    let firmware = ascii_trim(&data[64..72]);
    ControllerInfo {
        vid,
        ssvid,
        serial,
        model,
        firmware,
    }
}

fn ascii_trim(bytes: &[u8]) -> String {
    let s: String = bytes
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
