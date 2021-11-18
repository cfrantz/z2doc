use crate::description::nesfile;
use crate::dis::cpu6502::*;
use crate::dis::symtab::Symtab;
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
            let (operand, size) = match info.size {
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
                    symtab.maybe_put(segment.prgbank, operand, &format!("L{:04X}", operand));
                }
                AddressingMode::Relative => {
                    let mut disp = operand;
                    if disp & 0x80 == 0x80 {
                        disp |= 0xFF00;
                    }
                    let operand = (addr + 2).wrapping_add(disp);
                    symtab.maybe_put(segment.prgbank, operand, &format!("L{:04X}", operand));
                }
                _ => {}
            };
            self.instruction.push(Instruction {
                addr: addr,
                opcode: i,
                mnemonic: NAMES[i as usize],
                operand: operand,
                mode: info.mode,
            });
            addr += size;
        }
        Ok(())
    }

    fn to_text_one(&self, i: &Instruction, segment: &nesfile::Segment, symtab: &Symtab) -> String {
        let (operand, hex) = match i.mode {
            AddressingMode::Absolute
            | AddressingMode::AbsoluteX
            | AddressingMode::AbsoluteY
            | AddressingMode::Indirect => (
                symtab
                    .get(segment.prgbank, i.operand)
                    .unwrap_or(format!("${:04X}", i.operand)),
                format!(
                    "{:02X}{:02X}{:02X}",
                    i.opcode,
                    i.operand & 0xFF,
                    i.operand >> 8
                ),
            ),
            AddressingMode::Accumulator | AddressingMode::Implied => {
                (String::default(), format!("{:02X}", i.opcode))
            }
            AddressingMode::Immediate => (
                format!("${:02X}", i.operand),
                format!("{:02X}{:02X}", i.opcode, i.operand),
            ),
            AddressingMode::IndexedIndirect
            | AddressingMode::IndirectIndexed
            | AddressingMode::ZeroPage
            | AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY => (
                symtab
                    .get(segment.prgbank, i.operand)
                    .unwrap_or(format!("${:02X}", i.operand)),
                format!("{:02X}{:02X}", i.opcode, i.operand),
            ),
            AddressingMode::Relative => {
                let mut disp = i.operand;
                if disp & 0x80 == 0x80 {
                    disp |= 0xFF00;
                }
                let operand = (i.addr + 2).wrapping_add(disp);
                (
                    symtab
                        .get(segment.prgbank, operand)
                        .unwrap_or(format!("${:04X}", operand)),
                    format!("{:02X}{:02X}", i.opcode, i.operand),
                )
            }
        };

        let mut output = Vec::new();
        if let Some(mut symbol) = symtab.get(segment.prgbank, i.addr) {
            symbol.push_str(":");
            output.push(symbol);
        }
        let mut instruction = format!(
            "    {:<40}; {:04X} {:<6} ;",
            i.mnemonic.replace("@", &operand),
            i.addr,
            hex
        );

        if let Some(comment) = segment.address.get(&i.addr) {
            if !comment.header.is_empty() {
                for (j, line) in comment.header.split('\n').enumerate() {
                    output.insert(j, format!("; {}", line));
                }
            }
            if !comment.comment.is_empty() {
                instruction.push(' ');
                instruction.push_str(&comment.comment);
            }
            output.push(instruction);
            if !comment.footer.is_empty() {
                for line in comment.footer.split('\n') {
                    output.push(format!("; {}", line));
                }
            }
        } else {
            output.push(instruction);
        }
        output.join("\n")
    }

    pub fn to_text(&self, segment: &nesfile::Segment, symtab: &Symtab) {
        for i in &self.instruction {
            let text = self.to_text_one(i, segment, symtab);
            println!("{}", text);
        }
    }
}
