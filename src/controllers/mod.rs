pub mod innogrit;
pub mod marvell;
pub mod maxio;
pub mod phison;
pub mod rtl;
pub mod smi;
pub mod tenafe;

pub struct FlashBank {
    pub bank_num: u32,
    pub flash_id: [u8; 8],
}

pub struct FlashIdResult {
    pub controller_name: String,
    pub banks: Vec<FlashBank>,
}

fn is_bank_empty(data: &[u8]) -> bool {
    data.iter().all(|&b| b == 0x00) || data.iter().all(|&b| b == 0xFF)
}
