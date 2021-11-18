use crate::description::nesfile;
use crate::dis::symtab::Symtab;

#[derive(Debug, Default)]
pub struct DataBytesRange {
    pub start: u16,
    pub end: u16,
}

impl DataBytesRange {
    pub fn new(start: u16, end: u16) -> Self {
        DataBytesRange { start, end }
    }

    fn maybe_newline(nl: bool) {
        if nl {
            println!();
        }
    }

    pub fn to_text(&self, rom: &[u8], segment: &nesfile::Segment, symtab: &Symtab) {
        let mut n = 0;
        for addr in self.start..=self.end {
            if let Some(comment) = segment.address.get(&addr) {
                if !comment.header.is_empty() {
                    for line in comment.header.split('\n') {
                        Self::maybe_newline(n != 0);
                        println!("; {}", line);
                        n = 0;
                    }
                }
            }
            if let Some(symbol) = symtab.get(segment.prgbank, addr) {
                Self::maybe_newline(n != 0);
                println!("{}:", symbol);
                n = 0;
            }

            let fofs = segment.cpu_to_fofs(addr);
            if n % 8 == 0 {
                Self::maybe_newline(n != 0);
                print!("    .byte ${:02X}", rom[fofs]);
            } else {
                print!(",${:02X}", rom[fofs]);
            }
            n += 1;
        }
        println!();
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

    pub fn to_text(&self, rom: &[u8], segment: &nesfile::Segment, symtab: &Symtab) {
        for addr in (self.start..=self.end).step_by(2) {
            let fofs = segment.cpu_to_fofs(addr);
            if let Some(comment) = segment.address.get(&addr) {
                if !comment.header.is_empty() {
                    for line in comment.header.split('\n') {
                        println!("; {}", line);
                    }
                }
            }
            if let Some(symbol) = symtab.get(segment.prgbank, addr) {
                println!("{}:", symbol);
            }
            let value = (rom[fofs] as u16) | (rom[fofs + 1] as u16) << 8;
            if let Some(symbol) = symtab.get(segment.prgbank, value) {
                println!("    .word {}", symbol);
            } else {
                println!("    .word ${:04X}", value);
            }
        }
    }
}
