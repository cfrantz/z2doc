use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RegionInfo {
    Code(RangeInclusive<u16>),
    Bytes(RangeInclusive<u16>),
    Words(RangeInclusive<u16>),
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AnnotationInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_comment: Option<String>,
}

impl AnnotationInfo {
    pub fn is_empty(&self) -> bool {
        self.symbol.is_none() && self.comment.is_none() && self.block_comment.is_none()
    }
}

pub type SectionInfo = BTreeMap<u16, AnnotationInfo>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BankInfo {
    // Optional title for the bank (e.g. "West Hyrule")
    pub title: Option<String>,
    // Whether this bank is fixed at a specific CPU address (usually the high bank)
    #[serde(default)]
    pub is_fixed: bool,
    // List of code and data regions to aid disassembly.
    pub region: Vec<RegionInfo>,
    // Symbol and comment information for ROM code and data.
    pub address: SectionInfo,
    // The CPU address where this bank is typically mapped.
    pub mapped_at: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisassemblyInfo {
    // Unique project name for persistence keys (e.g. "zelda2")
    pub name: String,
    // Display title for the UI (e.g. "Zelda II: The Adventure of Link")
    pub title: String,
    // Symbol and comment information for RAM variables and global peripherals
    pub global: SectionInfo,
    // Information for ROM code and data. The u8 key is the bank index
    // based on the configured mapper window size (8K or 16K).
    pub bank: BTreeMap<u8, BankInfo>,
    // Mapper window size: 8 or 16
    pub mapper_window_size: u8,
    // The CPU address where this bank is mapped (e.g. 0xC000..=0xFFFF)
    pub mapper_fixed_range: Option<RangeInclusive<u16>>,
}

impl DisassemblyInfo {
    pub fn find_fixed_bank_id(&self) -> Option<u8> {
        for (id, bank) in &self.bank {
            if bank.is_fixed {
                return Some(*id);
            }
        }
        None
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeConfig {
    pub name: String,
    pub background: String,
    pub text: String,
    pub address: String,
    pub hex: String,
    pub instruction: String,
    pub opcode: String,
    pub comment: String,
    pub symbol: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct DisassemblyLine {
    pub address_label: String, // e.g., "$02:$8354"
    pub address: u16,
    pub bank: i16, // -1 for Global, 0-255 for PRG banks
    pub bytes: String,
    pub opcode: String,
    pub operand_prefix: String,
    pub operand_main: String,
    pub operand_suffix: String,
    pub operand_is_symbol: bool,
    pub symbol: Option<String>,
    pub comment: Option<String>,
    pub block_comment: Option<String>,
    pub target_bank: Option<u8>,
    pub target_address: Option<u16>,
}
