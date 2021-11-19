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

    fn maybe_newline(want: bool, n: usize, addr: u16, bytes: &[u8]) {
        if n == 0 || !want {
            return;
        }
        let n = n % 8;
        if n != 0 {
            for _ in 0..(8 - n) {
                print!("    ");
            }
        }
        print!("   ");

        let n = if n == 0 { 8 } else { n };
        println!(
            "; {:04X} {} ;",
            addr,
            std::str::from_utf8(&bytes[..n]).unwrap()
        )
    }

    fn maybe_printable(v: u8) -> u8 {
        // FIXME: nes games don't use ascii encoding.
        if v >= 32 && v < 127 {
            v
        } else {
            b'.'
        }
    }

    pub fn to_text(&self, rom: &[u8], segment: &nesfile::Segment, symtab: &Symtab) {
        let mut n = 0;
        let mut start = self.start;
        let mut bytes = [0u8; 8];

        for addr in self.start..=self.end {
            if let Some(comment) = segment.address.get(&addr) {
                if !comment.header.is_empty() {
                    Self::maybe_newline(n % 8 != 0, n, start, &bytes);
                    n = 0;
                    start = addr;
                    for line in comment.header.split('\n') {
                        println!("; {}", line);
                    }
                }
            }
            if let Some(symbol) = symtab.get_label(segment.prgbank, addr) {
                symtab.promote(segment.prgbank, addr, Some(&symbol));
                Self::maybe_newline(n % 8 != 0, n, start, &bytes);
                n = 0;
                start = addr;
                println!("{}:", symbol);
            }

            let fofs = segment.cpu_to_fofs(addr);
            let data = rom[fofs];
            bytes[n % 8] = Self::maybe_printable(data);
            if n % 8 == 0 {
                start = addr;
                print!("    .byte ${:02X}", data);
            } else {
                print!(",${:02X}", data);
            }
            n += 1;
            Self::maybe_newline(n % 8 == 0, n, start, &bytes);
        }
        Self::maybe_newline(n % 8 != 0, n, start, &bytes);
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
            if let Some(symbol) = symtab.get_label(segment.prgbank, addr) {
                symtab.promote(segment.prgbank, addr, Some(&symbol));
                println!("{}:", symbol);
            }
            let value = (rom[fofs] as u16) | (rom[fofs + 1] as u16) << 8;
            if let Some(symbol) = symtab.get(segment.prgbank, value) {
                symtab.promote(segment.prgbank, value, Some(&symbol));
                println!("    .word {}", symbol);
            } else {
                println!("    .word ${:04X}", value);
            }
        }
    }
}
