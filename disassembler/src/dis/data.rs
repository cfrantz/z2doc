use itertools::Itertools;
use std::iter::FromIterator;

use crate::description::nesfile;
use crate::dis::symtab::Symtab;
use crate::output::{self, Format};

#[derive(Debug, Default)]
pub struct DataBytesRange {
    pub start: u16,
    pub end: u16,
}

impl DataBytesRange {
    pub fn new(start: u16, end: u16) -> Self {
        DataBytesRange { start, end }
    }

    fn as_ascii(v: &u8) -> char {
        // FIXME: nes games don't use ascii encoding.
        if *v >= 32 && *v < 127 {
            *v as char
        } else {
            '.'
        }
    }

    pub fn to_text(
        &self,
        fmt: Format,
        rom: &[u8],
        segment: &nesfile::Segment,
        symtab: &Symtab,
    ) -> Vec<String> {
        let mut ret = Vec::new();
        let mut addr = self.start;
        loop {
            let mut end = addr;
            for i in (addr + 1)..=std::cmp::min(addr + 7, self.end) {
                if symtab.get_label(segment.prgbank, i).is_some() {
                    break;
                }
                end = i;
            }
            let a = segment.cpu_to_fofs(addr);
            let b = segment.cpu_to_fofs(end);
            let bytes = &rom[a..=b];
            let operand = bytes.iter().map(|b| format!("${:02X}", b)).join(",");
            let hexdump = String::from_iter(bytes.iter().map(Self::as_ascii));
            let label = symtab.get_label(segment.prgbank, addr);

            if let Some(comment) = segment.address.get(&addr) {
                ret.extend(output::commentblock(fmt, &comment.header));
                if let Some(l) = &label {
                    symtab.promote(segment.prgbank, addr, Some(l));
                    ret.push(output::label(fmt, l));
                }
                ret.push(output::instruction(
                    fmt,
                    ".byte @",
                    &operand,
                    None,
                    addr,
                    &hexdump,
                    &comment.comment,
                ));
                ret.extend(output::commentblock(fmt, &comment.footer));
            } else {
                if let Some(l) = &label {
                    symtab.promote(segment.prgbank, addr, Some(l));
                    ret.push(output::label(fmt, l));
                }
                ret.push(output::instruction(
                    fmt, ".byte @", &operand, None, addr, &hexdump, "",
                ));
            }
            if end == self.end {
                break;
            }
            addr = end.wrapping_add(1);
        }
        ret
    }
}

#[derive(Debug, Default)]
pub struct DataWordsRange {
    pub start: u16,
    pub end: u16,
}

impl DataWordsRange {
    pub fn new(start: u16, end: u16) -> Self {
        DataWordsRange { start, end }
    }

    pub fn to_text(
        &self,
        fmt: Format,
        rom: &[u8],
        segment: &nesfile::Segment,
        symtab: &Symtab,
    ) -> Vec<String> {
        let mut ret = Vec::new();
        for addr in (self.start..=self.end).step_by(2) {
            let fofs = segment.cpu_to_fofs(addr);
            let value = (rom[fofs] as u16) | (rom[fofs + 1] as u16) << 8;
            let operand = format!("${:04X}", value);
            let hex = format!("{:02X}{:02X}", rom[fofs], rom[fofs + 1]);
            let symbol = symtab.get(segment.prgbank, value);
            let label = symtab.get_label(segment.prgbank, addr);

            if let Some(comment) = segment.address.get(&addr) {
                ret.extend(output::commentblock(fmt, &comment.header));
                if let Some(l) = &label {
                    symtab.promote(segment.prgbank, addr, Some(l));
                    ret.push(output::label(fmt, l));
                }
                ret.push(output::instruction(
                    fmt,
                    ".word @",
                    &operand,
                    symbol.as_ref().map(String::as_str),
                    addr,
                    &hex,
                    &comment.comment,
                ));
                ret.extend(output::commentblock(fmt, &comment.footer));
            } else {
                if let Some(l) = &label {
                    symtab.promote(segment.prgbank, addr, Some(l));
                    ret.push(output::label(fmt, l));
                }
                ret.push(output::instruction(
                    fmt,
                    ".word @",
                    &operand,
                    symbol.as_ref().map(String::as_str),
                    addr,
                    &hex,
                    "",
                ));
            }
        }
        ret
    }
}
