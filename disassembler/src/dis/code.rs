use crate::description::nesfile;
use crate::dis::cpu6502::*;
use crate::dis::symtab::Symtab;
use crate::output::{self, Format};
use anyhow::Result;

#[derive(Debug, Default)]
pub struct CodeRange {
    pub start: u16,
    pub end: u16,
    pub instruction: Vec<Instruction>,
}

#[derive(Debug)]
pub struct Instruction {
    pub addr: u16,
    pub opcode: u8,
    pub mnemonic: &'static str,
    pub operand: u16,
    pub mode: AddressingMode,
}

impl CodeRange {
    pub fn new(start: u16, end: u16) -> Self {
        CodeRange {
            start: start,
            end: end,
            instruction: Vec::new(),
        }
    }

    pub fn disassemble(
        &mut self,
        rom: &[u8],
        segment: &nesfile::Segment,
        symtab: &Symtab,
    ) -> Result<()> {
        let mut addr = self.start;
        while addr <= self.end {
            let fofs = segment.cpu_to_fofs(addr);
            let i = rom[fofs];
            let info = &INFO[i as usize];
            let (operand, mut size) = match info.size {
                0 => {
                    log::warn!("Illegal instruction at {} ${:04x}", segment.name, addr);
                    (0, 1)
                }
                1 => (0, 1),
                2 => (rom[fofs + 1] as u16, 2),
                3 => ((rom[fofs + 1] as u16) | (rom[fofs + 2] as u16) << 8, 3),
                _ => panic!("Bad size info in cpu6502 for {:02x}", i),
            };
            match info.mode {
                AddressingMode::Absolute
                | AddressingMode::AbsoluteX
                | AddressingMode::AbsoluteY
                | AddressingMode::Indirect => {
                    symtab.synthetic_put(segment.prgbank, operand, &format!("L{:04X}", operand));
                }
                AddressingMode::Relative => {
                    let mut disp = operand;
                    if disp & 0x80 == 0x80 {
                        disp |= 0xFF00;
                    }
                    let operand = (addr + 2).wrapping_add(disp);
                    symtab.synthetic_put(segment.prgbank, operand, &format!("L{:04X}", operand));
                }
                _ => {}
            };
            if i == 0x2C
                && (symtab.get(segment.prgbank, addr + 1).is_some()
                    || symtab.get(segment.prgbank, addr + 2).is_some())
            {
                self.instruction.push(Instruction {
                    addr: addr,
                    opcode: i,
                    mnemonic: ".byte $2C ; BIT used as a skip",
                    operand: operand,
                    mode: info.mode,
                });
                size = 1;
            } else {
                self.instruction.push(Instruction {
                    addr: addr,
                    opcode: i,
                    mnemonic: NAMES[i as usize],
                    operand: operand,
                    mode: info.mode,
                });
            }
            let next = addr.wrapping_add(size);
            if next < addr {
                // Address wrapped, we are done.
                break;
            }
            addr = next;
        }
        Ok(())
    }

    fn to_text_one(
        &self,
        fmt: Format,
        i: &Instruction,
        segment: &nesfile::Segment,
        symtab: &Symtab,
    ) -> Vec<String> {
        let (operand, symbol, hex) = match i.mode {
            AddressingMode::Absolute | AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => {
                let symbol = if i.mnemonic.starts_with("ST") && i.operand >= 0x8000 {
                    // Hack: stores >= 0x8000 are usually mapper hardware
                    symtab.get(None, i.operand)
                } else {
                    symtab.get_offset(segment.prgbank, i.operand)
                };
                symtab.promote(
                    segment.prgbank,
                    i.operand,
                    symbol.as_ref().map(String::as_str),
                );
                (
                    // CA65 uses "a:" to represent an absolute address override.
                    format!(
                        "{}${:04X}",
                        if i.operand < 256 { "a:" } else { "" },
                        i.operand
                    ),
                    symbol.map(|s| format!("{}{}", if i.operand < 256 { "a:" } else { "" }, s)),
                    format!(
                        "{:02X}{:02X}{:02X}",
                        i.opcode,
                        i.operand & 0xFF,
                        i.operand >> 8
                    ),
                )
            }
            AddressingMode::Indirect => {
                let symbol = symtab.get_offset(segment.prgbank, i.operand);
                symtab.promote(
                    segment.prgbank,
                    i.operand,
                    symbol.as_ref().map(String::as_str),
                );
                (
                    format!("${:04X}", i.operand),
                    symbol,
                    format!(
                        "{:02X}{:02X}{:02X}",
                        i.opcode,
                        i.operand & 0xFF,
                        i.operand >> 8
                    ),
                )
            }

            AddressingMode::Accumulator | AddressingMode::Implied => {
                (String::default(), None, format!("{:02X}", i.opcode))
            }
            AddressingMode::Immediate => (
                format!("${:02X}", i.operand),
                None,
                format!("{:02X}{:02X}", i.opcode, i.operand),
            ),
            AddressingMode::IndexedIndirect
            | AddressingMode::IndirectIndexed
            | AddressingMode::ZeroPage
            | AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY => {
                let symbol = symtab.get_offset(segment.prgbank, i.operand);
                symtab.promote(
                    segment.prgbank,
                    i.operand,
                    symbol.as_ref().map(String::as_str),
                );
                (
                    format!("${:02X}", i.operand),
                    symbol,
                    format!("{:02X}{:02X}", i.opcode, i.operand),
                )
            }
            AddressingMode::Relative => {
                let mut disp = i.operand;
                if disp & 0x80 == 0x80 {
                    disp |= 0xFF00;
                }
                let operand = (i.addr + 2).wrapping_add(disp);
                let symbol = symtab.get_label(segment.prgbank, operand);
                symtab.promote(
                    segment.prgbank,
                    operand,
                    symbol.as_ref().map(String::as_str),
                );
                (
                    format!("${:04X}", operand),
                    symbol,
                    format!("{:02X}{:02X}", i.opcode, i.operand),
                )
            }
        };

        let mut ret = Vec::new();
        if let Some(symbol) = symtab.get_label(segment.prgbank, i.addr) {
            ret.push(output::label(fmt, &symbol));
        }
        if let Some(comment) = segment.address.get(&i.addr) {
            ret.extend(output::commentblock(fmt, &comment.header));
            ret.push(output::instruction(
                fmt,
                i.mnemonic,
                &operand,
                symbol.as_ref().map(String::as_str),
                i.addr,
                &hex,
                &comment.comment,
            ));
            ret.extend(output::commentblock(fmt, &comment.footer));
        } else {
            ret.push(output::instruction(
                fmt,
                i.mnemonic,
                &operand,
                symbol.as_ref().map(String::as_str),
                i.addr,
                &hex,
                "",
            ));
        }
        ret
    }

    pub fn to_text(&self, fmt: Format, segment: &nesfile::Segment, symtab: &Symtab) -> Vec<String> {
        let mut ret = Vec::new();
        for i in &self.instruction {
            ret.extend(self.to_text_one(fmt, i, segment, symtab));
        }
        ret
    }
}
