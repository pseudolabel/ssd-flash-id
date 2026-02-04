pub mod ata_identify_fid;
pub mod innogrit;
pub mod jm_sata;
pub mod marvell;
pub mod maxio;
pub mod phison;
pub mod rtl;
pub mod rtl_sata;
pub mod sandforce;
pub mod smi;
pub mod smi_sata;
pub mod tenafe;
pub mod yeestor;

#[derive(Clone)]
pub struct FlashBank {
    pub bank_num: u32,
    pub flash_id: [u8; 8],
}

#[derive(Clone)]
pub struct FlashIdResult {
    pub controller_name: String,
    pub banks: Vec<FlashBank>,
}

fn is_bank_empty(data: &[u8]) -> bool {
    data.iter().all(|&b| b == 0x00) || data.iter().all(|&b| b == 0xFF)
}
