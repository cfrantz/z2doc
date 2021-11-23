use crate::description::nesfile;
use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::dis::code::CodeRange;
use crate::dis::data::{DataBytesRange, DataWordsRange};
use crate::dis::symtab::Symtab;
use crate::output::{self, Format};

#[derive(Debug)]
pub enum Range {
    Code(CodeRange),
    Bytes(DataBytesRange),
    Words(DataWordsRange),
}

impl Range {
    pub fn bytes(start: u16, end: u16) -> Self {
        Range::Bytes(DataBytesRange::new(start, end))
    }
    pub fn words(start: u16, end: u16) -> Self {
        Range::Words(DataWordsRange::new(start, end))
    }

    pub fn to_text(
        &self,
        fmt: Format,
        rom: &[u8],
        segment: &nesfile::Segment,
        symtab: &Symtab,
    ) -> Vec<String> {
        match self {
            Range::Code(x) => x.to_text(fmt, segment, symtab),
            Range::Bytes(x) => x.to_text(fmt, rom, segment, symtab),
            Range::Words(x) => x.to_text(fmt, rom, segment, symtab),
        }
    }
}

#[derive(Debug)]
pub struct Segment {
    range: Vec<Range>,
}

impl Default for Segment {
    fn default() -> Self {
        Segment { range: Vec::new() }
    }
}

#[derive(Debug, Default)]
pub struct Rom {
    rom: Vec<u8>,
    segment: Vec<Segment>,
    symtab: Symtab,
}

impl Rom {
    pub fn new(romfile: &Path) -> Result<Self> {
        let mut rom = Rom::default();
        let mut file = File::open(romfile)?;
        file.read_to_end(&mut rom.rom)?;
        Ok(rom)
    }

    fn process_segment(&mut self, _info: &nesfile::NesFile, s: &nesfile::Segment) -> Result<()> {
        let mut seg = Segment::default();
        let mut addr = s.fofs_to_cpu(*s.file_range.start()) as u32;
        // cpu addr is a 16-bit word, but we want a half-open range [start, end).
        let end = s.fofs_to_cpu(*s.file_range.end()) as u32 + 1;
        log::info!("segment: {:?} start={:x?} end={:x?}", s.name, addr, end);
        while addr < end {
            let range = s.get_range(addr as u16)?;
            if let Some(r) = range {
                addr = match r {
                    nesfile::DataRange::Code(a, b) => {
                        let mut code = CodeRange::new(*a, *b);
                        code.disassemble(&self.rom, s, &self.symtab)?;
                        seg.range.push(Range::Code(code));
                        *b as u32 + 1
                    }
                    nesfile::DataRange::Bytes(a, b) => {
                        seg.range.push(Range::bytes(*a, *b));
                        *b as u32 + 1
                    }
                    nesfile::DataRange::Words(a, b) => {
                        seg.range.push(Range::words(*a, *b));
                        *b as u32 + 1
                    }
                };
            } else {
                let start = addr;
                while addr < end && s.get_range(addr as u16)?.is_none() {
                    addr += 1;
                }
                seg.range
                    .push(Range::bytes(start as u16, (addr - 1) as u16));
            }
        }
        self.segment.push(seg);
        Ok(())
    }

    fn process_symtab(&mut self, info: &nesfile::NesFile) -> Result<()> {
        let last = info.segment.last().ok_or(anyhow!("No last segment"))?;
        self.symtab
            .set_highbank(last.cpu_range.clone(), last.prgbank);

        for (addr, symbol) in info.global_symbols.iter() {
            self.symtab.put(None, *addr, &symbol);
        }
        for segment in info.segment.iter() {
            for (addr, comment) in segment.address.iter() {
                if !comment.symbol.is_empty() {
                    self.symtab.put(segment.prgbank, *addr, &comment.symbol);
                }
            }
        }
        Ok(())
    }

    pub fn process(&mut self, info: &nesfile::NesFile) -> Result<()> {
        self.process_symtab(info)?;
        for s in info.segment.iter() {
            self.process_segment(info, s)?;
        }
        Ok(())
    }

    pub fn to_text(&self, fmt: Format, info: &nesfile::NesFile) -> Vec<String> {
        let mut ret = Vec::new();
        ret.extend(output::commentblock(fmt, &info.header));
        for (addr, sym) in self.symtab.get_globals().iter() {
            ret.push(output::equate(fmt, sym, &format!("${:04x}", addr)));
        }
        for (seg, nesseg) in self.segment.iter().zip(&info.segment) {
            ret.extend(output::commentblock(fmt, &nesseg.header));
            ret.push(output::directive(
                fmt,
                &format!(".segment \"{}\"", nesseg.name),
            ));
            for range in &seg.range {
                ret.extend(range.to_text(fmt, &self.rom, nesseg, &self.symtab));
            }
            ret.extend(output::commentblock(fmt, &nesseg.footer));
        }
        ret
    }
}
