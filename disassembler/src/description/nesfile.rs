use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Range;
use std::path::PathBuf;

use crate::description::range;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NesFile {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nesfile: Option<PathBuf>,
    #[serde(default)]
    pub global_symbols: HashMap<u16, String>,
    #[serde(default)]
    pub segment: Vec<Segment>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Segment {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prgbank: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chrbank: Option<i16>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub header: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub footer: String,
    #[serde(skip_serializing_if = "Range::is_empty", with = "range")]
    pub file_range: Range<usize>,
    #[serde(skip_serializing_if = "Range::is_empty", with = "range")]
    pub cpu_range: Range<usize>,
    #[serde(default)]
    pub range: Vec<DataRange>,
    #[serde(default)]
    pub address: HashMap<u16, Comment>,
}

impl Segment {
    pub fn is_bytes(&self, address: u16) -> bool {
        self.range.iter().any(|x| x.is_bytes(address))
    }
    pub fn is_words(&self, address: u16) -> bool {
        self.range.iter().any(|x| x.is_words(address))
    }
    pub fn is_code(&self, address: u16) -> bool {
        self.range.iter().any(|x| x.is_code(address))
    }
    pub fn get_range(&self, address: u16) -> Result<Option<&DataRange>> {
        let mut it = self.range.iter().filter(|x| x.contains(address));
        let val = it.next();
        if it.count() == 0 {
            Ok(val)
        } else {
            Err(anyhow!("Overlapping ranges for address 0x{:04x}", address))
        }
    }
    pub fn fofs_to_cpu(&self, fofs: usize) -> u16 {
        (fofs + self.cpu_range.start - self.file_range.start) as u16
    }
    pub fn cpu_to_fofs(&self, address: u16) -> usize {
        (address as usize) + self.file_range.start - self.cpu_range.start
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DataRange {
    Bytes(u16, u16),
    Words(u16, u16),
    Code(u16, u16),
}

impl Default for DataRange {
    fn default() -> Self {
        DataRange::Bytes(0, 0)
    }
}

impl DataRange {
    pub fn contains(&self, address: u16) -> bool {
        match self {
            DataRange::Code(start, end)
            | DataRange::Bytes(start, end)
            | DataRange::Words(start, end) => address >= *start && address < *end,
        }
    }
    pub fn is_code(&self, address: u16) -> bool {
        if let DataRange::Code(start, end) = self {
            address >= *start && address < *end
        } else {
            false
        }
    }
    pub fn is_bytes(&self, address: u16) -> bool {
        if let DataRange::Bytes(start, end) = self {
            address >= *start && address < *end
        } else {
            false
        }
    }
    pub fn is_words(&self, address: u16) -> bool {
        if let DataRange::Words(start, end) = self {
            address >= *start && address < *end
        } else {
            false
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Comment {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub comment: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub symbol: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub header: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub footer: String,
}
