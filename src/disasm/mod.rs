use crate::models::{AnnotationInfo, DisassemblyInfo, DisassemblyLine, RegionInfo};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirect,
    IndirectIndexed,
}

impl AddressingMode {
    pub fn operand_length(&self) -> u16 {
        match self {
            AddressingMode::Implied | AddressingMode::Accumulator => 0,
            AddressingMode::Immediate
            | AddressingMode::ZeroPage
            | AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY
            | AddressingMode::Relative
            | AddressingMode::IndexedIndirect
            | AddressingMode::IndirectIndexed => 1,
            AddressingMode::Absolute
            | AddressingMode::AbsoluteX
            | AddressingMode::AbsoluteY
            | AddressingMode::Indirect => 2,
        }
    }
}

pub struct Instruction {
    pub mnemonic: &'static str,
    pub mode: AddressingMode,
}

pub const OPCODES: [Option<Instruction>; 256] = {
    let mut table: [Option<Instruction>; 256] = [const { None }; 256];

    // ADC
    table[0x69] = Some(Instruction { mnemonic: "ADC", mode: AddressingMode::Immediate });
    table[0x65] = Some(Instruction { mnemonic: "ADC", mode: AddressingMode::ZeroPage });
    table[0x75] = Some(Instruction { mnemonic: "ADC", mode: AddressingMode::ZeroPageX });
    table[0x6D] = Some(Instruction { mnemonic: "ADC", mode: AddressingMode::Absolute });
    table[0x7D] = Some(Instruction { mnemonic: "ADC", mode: AddressingMode::AbsoluteX });
    table[0x79] = Some(Instruction { mnemonic: "ADC", mode: AddressingMode::AbsoluteY });
    table[0x61] = Some(Instruction { mnemonic: "ADC", mode: AddressingMode::IndexedIndirect });
    table[0x71] = Some(Instruction { mnemonic: "ADC", mode: AddressingMode::IndirectIndexed });

    // AND
    table[0x29] = Some(Instruction { mnemonic: "AND", mode: AddressingMode::Immediate });
    table[0x25] = Some(Instruction { mnemonic: "AND", mode: AddressingMode::ZeroPage });
    table[0x35] = Some(Instruction { mnemonic: "AND", mode: AddressingMode::ZeroPageX });
    table[0x2D] = Some(Instruction { mnemonic: "AND", mode: AddressingMode::Absolute });
    table[0x3D] = Some(Instruction { mnemonic: "AND", mode: AddressingMode::AbsoluteX });
    table[0x39] = Some(Instruction { mnemonic: "AND", mode: AddressingMode::AbsoluteY });
    table[0x21] = Some(Instruction { mnemonic: "AND", mode: AddressingMode::IndexedIndirect });
    table[0x31] = Some(Instruction { mnemonic: "AND", mode: AddressingMode::IndirectIndexed });

    // ASL
    table[0x0A] = Some(Instruction { mnemonic: "ASL", mode: AddressingMode::Accumulator });
    table[0x06] = Some(Instruction { mnemonic: "ASL", mode: AddressingMode::ZeroPage });
    table[0x16] = Some(Instruction { mnemonic: "ASL", mode: AddressingMode::ZeroPageX });
    table[0x0E] = Some(Instruction { mnemonic: "ASL", mode: AddressingMode::Absolute });
    table[0x1E] = Some(Instruction { mnemonic: "ASL", mode: AddressingMode::AbsoluteX });

    // BCC
    table[0x90] = Some(Instruction { mnemonic: "BCC", mode: AddressingMode::Relative });

    // BCS
    table[0xB0] = Some(Instruction { mnemonic: "BCS", mode: AddressingMode::Relative });

    // BEQ
    table[0xF0] = Some(Instruction { mnemonic: "BEQ", mode: AddressingMode::Relative });

    // BIT
    table[0x24] = Some(Instruction { mnemonic: "BIT", mode: AddressingMode::ZeroPage });
    table[0x2C] = Some(Instruction { mnemonic: "BIT", mode: AddressingMode::Absolute });

    // BMI
    table[0x30] = Some(Instruction { mnemonic: "BMI", mode: AddressingMode::Relative });

    // BNE
    table[0xD0] = Some(Instruction { mnemonic: "BNE", mode: AddressingMode::Relative });

    // BPL
    table[0x10] = Some(Instruction { mnemonic: "BPL", mode: AddressingMode::Relative });

    // BRK
    table[0x00] = Some(Instruction { mnemonic: "BRK", mode: AddressingMode::Implied });

    // BVC
    table[0x50] = Some(Instruction { mnemonic: "BVC", mode: AddressingMode::Relative });

    // BVS
    table[0x70] = Some(Instruction { mnemonic: "BVS", mode: AddressingMode::Relative });

    // CLC
    table[0x18] = Some(Instruction { mnemonic: "CLC", mode: AddressingMode::Implied });

    // CLD
    table[0xD8] = Some(Instruction { mnemonic: "CLD", mode: AddressingMode::Implied });

    // CLI
    table[0x58] = Some(Instruction { mnemonic: "CLI", mode: AddressingMode::Implied });

    // CLV
    table[0xB8] = Some(Instruction { mnemonic: "CLV", mode: AddressingMode::Implied });

    // CMP
    table[0xC9] = Some(Instruction { mnemonic: "CMP", mode: AddressingMode::Immediate });
    table[0xC5] = Some(Instruction { mnemonic: "CMP", mode: AddressingMode::ZeroPage });
    table[0xD5] = Some(Instruction { mnemonic: "CMP", mode: AddressingMode::ZeroPageX });
    table[0xCD] = Some(Instruction { mnemonic: "CMP", mode: AddressingMode::Absolute });
    table[0xDD] = Some(Instruction { mnemonic: "CMP", mode: AddressingMode::AbsoluteX });
    table[0xD9] = Some(Instruction { mnemonic: "CMP", mode: AddressingMode::AbsoluteY });
    table[0xC1] = Some(Instruction { mnemonic: "CMP", mode: AddressingMode::IndexedIndirect });
    table[0xD1] = Some(Instruction { mnemonic: "CMP", mode: AddressingMode::IndirectIndexed });

    // CPX
    table[0xE0] = Some(Instruction { mnemonic: "CPX", mode: AddressingMode::Immediate });
    table[0xE4] = Some(Instruction { mnemonic: "CPX", mode: AddressingMode::ZeroPage });
    table[0xEC] = Some(Instruction { mnemonic: "CPX", mode: AddressingMode::Absolute });

    // CPY
    table[0xC0] = Some(Instruction { mnemonic: "CPY", mode: AddressingMode::Immediate });
    table[0xC4] = Some(Instruction { mnemonic: "CPY", mode: AddressingMode::ZeroPage });
    table[0xCC] = Some(Instruction { mnemonic: "CPY", mode: AddressingMode::Absolute });

    // DEC
    table[0xC6] = Some(Instruction { mnemonic: "DEC", mode: AddressingMode::ZeroPage });
    table[0xD6] = Some(Instruction { mnemonic: "DEC", mode: AddressingMode::ZeroPageX });
    table[0xCE] = Some(Instruction { mnemonic: "DEC", mode: AddressingMode::Absolute });
    table[0xDE] = Some(Instruction { mnemonic: "DEC", mode: AddressingMode::AbsoluteX });

    // DEX
    table[0xCA] = Some(Instruction { mnemonic: "DEX", mode: AddressingMode::Implied });

    // DEY
    table[0x88] = Some(Instruction { mnemonic: "DEY", mode: AddressingMode::Implied });

    // EOR
    table[0x49] = Some(Instruction { mnemonic: "EOR", mode: AddressingMode::Immediate });
    table[0x45] = Some(Instruction { mnemonic: "EOR", mode: AddressingMode::ZeroPage });
    table[0x55] = Some(Instruction { mnemonic: "EOR", mode: AddressingMode::ZeroPageX });
    table[0x4D] = Some(Instruction { mnemonic: "EOR", mode: AddressingMode::Absolute });
    table[0x5D] = Some(Instruction { mnemonic: "EOR", mode: AddressingMode::AbsoluteX });
    table[0x59] = Some(Instruction { mnemonic: "EOR", mode: AddressingMode::AbsoluteY });
    table[0x41] = Some(Instruction { mnemonic: "EOR", mode: AddressingMode::IndexedIndirect });
    table[0x51] = Some(Instruction { mnemonic: "EOR", mode: AddressingMode::IndirectIndexed });

    // INC
    table[0xE6] = Some(Instruction { mnemonic: "INC", mode: AddressingMode::ZeroPage });
    table[0xF6] = Some(Instruction { mnemonic: "INC", mode: AddressingMode::ZeroPageX });
    table[0xEE] = Some(Instruction { mnemonic: "INC", mode: AddressingMode::Absolute });
    table[0xFE] = Some(Instruction { mnemonic: "INC", mode: AddressingMode::AbsoluteX });

    // INX
    table[0xE8] = Some(Instruction { mnemonic: "INX", mode: AddressingMode::Implied });

    // INY
    table[0xC8] = Some(Instruction { mnemonic: "INY", mode: AddressingMode::Implied });

    // JMP
    table[0x4C] = Some(Instruction { mnemonic: "JMP", mode: AddressingMode::Absolute });
    table[0x6C] = Some(Instruction { mnemonic: "JMP", mode: AddressingMode::Indirect });

    // JSR
    table[0x20] = Some(Instruction { mnemonic: "JSR", mode: AddressingMode::Absolute });

    // LDA
    table[0xA9] = Some(Instruction { mnemonic: "LDA", mode: AddressingMode::Immediate });
    table[0xA5] = Some(Instruction { mnemonic: "LDA", mode: AddressingMode::ZeroPage });
    table[0xB5] = Some(Instruction { mnemonic: "LDA", mode: AddressingMode::ZeroPageX });
    table[0xAD] = Some(Instruction { mnemonic: "LDA", mode: AddressingMode::Absolute });
    table[0xBD] = Some(Instruction { mnemonic: "LDA", mode: AddressingMode::AbsoluteX });
    table[0xB9] = Some(Instruction { mnemonic: "LDA", mode: AddressingMode::AbsoluteY });
    table[0xA1] = Some(Instruction { mnemonic: "LDA", mode: AddressingMode::IndexedIndirect });
    table[0xB1] = Some(Instruction { mnemonic: "LDA", mode: AddressingMode::IndirectIndexed });

    // LDX
    table[0xA2] = Some(Instruction { mnemonic: "LDX", mode: AddressingMode::Immediate });
    table[0xA6] = Some(Instruction { mnemonic: "LDX", mode: AddressingMode::ZeroPage });
    table[0xB6] = Some(Instruction { mnemonic: "LDX", mode: AddressingMode::ZeroPageY });
    table[0xAE] = Some(Instruction { mnemonic: "LDX", mode: AddressingMode::Absolute });
    table[0xBE] = Some(Instruction { mnemonic: "LDX", mode: AddressingMode::AbsoluteY });

    // LDY
    table[0xA0] = Some(Instruction { mnemonic: "LDY", mode: AddressingMode::Immediate });
    table[0xA4] = Some(Instruction { mnemonic: "LDY", mode: AddressingMode::ZeroPage });
    table[0xB4] = Some(Instruction { mnemonic: "LDY", mode: AddressingMode::ZeroPageX });
    table[0xAC] = Some(Instruction { mnemonic: "LDY", mode: AddressingMode::Absolute });
    table[0xBC] = Some(Instruction { mnemonic: "LDY", mode: AddressingMode::AbsoluteX });

    // LSR
    table[0x4A] = Some(Instruction { mnemonic: "LSR", mode: AddressingMode::Accumulator });
    table[0x46] = Some(Instruction { mnemonic: "LSR", mode: AddressingMode::ZeroPage });
    table[0x56] = Some(Instruction { mnemonic: "LSR", mode: AddressingMode::ZeroPageX });
    table[0x4E] = Some(Instruction { mnemonic: "LSR", mode: AddressingMode::Absolute });
    table[0x5E] = Some(Instruction { mnemonic: "LSR", mode: AddressingMode::AbsoluteX });

    // NOP
    table[0xEA] = Some(Instruction { mnemonic: "NOP", mode: AddressingMode::Implied });

    // ORA
    table[0x09] = Some(Instruction { mnemonic: "ORA", mode: AddressingMode::Immediate });
    table[0x05] = Some(Instruction { mnemonic: "ORA", mode: AddressingMode::ZeroPage });
    table[0x15] = Some(Instruction { mnemonic: "ORA", mode: AddressingMode::ZeroPageX });
    table[0x0D] = Some(Instruction { mnemonic: "ORA", mode: AddressingMode::Absolute });
    table[0x1D] = Some(Instruction { mnemonic: "ORA", mode: AddressingMode::AbsoluteX });
    table[0x19] = Some(Instruction { mnemonic: "ORA", mode: AddressingMode::AbsoluteY });
    table[0x01] = Some(Instruction { mnemonic: "ORA", mode: AddressingMode::IndexedIndirect });
    table[0x11] = Some(Instruction { mnemonic: "ORA", mode: AddressingMode::IndirectIndexed });

    // PHA
    table[0x48] = Some(Instruction { mnemonic: "PHA", mode: AddressingMode::Implied });

    // PHP
    table[0x08] = Some(Instruction { mnemonic: "PHP", mode: AddressingMode::Implied });

    // PLA
    table[0x68] = Some(Instruction { mnemonic: "PLA", mode: AddressingMode::Implied });

    // PLP
    table[0x28] = Some(Instruction { mnemonic: "PLP", mode: AddressingMode::Implied });

    // ROL
    table[0x2A] = Some(Instruction { mnemonic: "ROL", mode: AddressingMode::Accumulator });
    table[0x26] = Some(Instruction { mnemonic: "ROL", mode: AddressingMode::ZeroPage });
    table[0x36] = Some(Instruction { mnemonic: "ROL", mode: AddressingMode::ZeroPageX });
    table[0x2E] = Some(Instruction { mnemonic: "ROL", mode: AddressingMode::Absolute });
    table[0x3E] = Some(Instruction { mnemonic: "ROL", mode: AddressingMode::AbsoluteX });

    // ROR
    table[0x6A] = Some(Instruction { mnemonic: "ROR", mode: AddressingMode::Accumulator });
    table[0x66] = Some(Instruction { mnemonic: "ROR", mode: AddressingMode::ZeroPage });
    table[0x76] = Some(Instruction { mnemonic: "ROR", mode: AddressingMode::ZeroPageX });
    table[0x6E] = Some(Instruction { mnemonic: "ROR", mode: AddressingMode::Absolute });
    table[0x7E] = Some(Instruction { mnemonic: "ROR", mode: AddressingMode::AbsoluteX });

    // RTI
    table[0x40] = Some(Instruction { mnemonic: "RTI", mode: AddressingMode::Implied });

    // RTS
    table[0x60] = Some(Instruction { mnemonic: "RTS", mode: AddressingMode::Implied });

    // SBC
    table[0xE9] = Some(Instruction { mnemonic: "SBC", mode: AddressingMode::Immediate });
    table[0xE5] = Some(Instruction { mnemonic: "SBC", mode: AddressingMode::ZeroPage });
    table[0xF5] = Some(Instruction { mnemonic: "SBC", mode: AddressingMode::ZeroPageX });
    table[0xED] = Some(Instruction { mnemonic: "SBC", mode: AddressingMode::Absolute });
    table[0xFD] = Some(Instruction { mnemonic: "SBC", mode: AddressingMode::AbsoluteX });
    table[0xF9] = Some(Instruction { mnemonic: "SBC", mode: AddressingMode::AbsoluteY });
    table[0xE1] = Some(Instruction { mnemonic: "SBC", mode: AddressingMode::IndexedIndirect });
    table[0xF1] = Some(Instruction { mnemonic: "SBC", mode: AddressingMode::IndirectIndexed });

    // SEC
    table[0x38] = Some(Instruction { mnemonic: "SEC", mode: AddressingMode::Implied });

    // SED
    table[0xF8] = Some(Instruction { mnemonic: "SED", mode: AddressingMode::Implied });

    // SEI
    table[0x78] = Some(Instruction { mnemonic: "SEI", mode: AddressingMode::Implied });

    // STA
    table[0x85] = Some(Instruction { mnemonic: "STA", mode: AddressingMode::ZeroPage });
    table[0x95] = Some(Instruction { mnemonic: "STA", mode: AddressingMode::ZeroPageX });
    table[0x8D] = Some(Instruction { mnemonic: "STA", mode: AddressingMode::Absolute });
    table[0x9D] = Some(Instruction { mnemonic: "STA", mode: AddressingMode::AbsoluteX });
    table[0x99] = Some(Instruction { mnemonic: "STA", mode: AddressingMode::AbsoluteY });
    table[0x81] = Some(Instruction { mnemonic: "STA", mode: AddressingMode::IndexedIndirect });
    table[0x91] = Some(Instruction { mnemonic: "STA", mode: AddressingMode::IndirectIndexed });

    // STX
    table[0x86] = Some(Instruction { mnemonic: "STX", mode: AddressingMode::ZeroPage });
    table[0x96] = Some(Instruction { mnemonic: "STX", mode: AddressingMode::ZeroPageY });
    table[0x8E] = Some(Instruction { mnemonic: "STX", mode: AddressingMode::Absolute });

    // STY
    table[0x84] = Some(Instruction { mnemonic: "STY", mode: AddressingMode::ZeroPage });
    table[0x94] = Some(Instruction { mnemonic: "STY", mode: AddressingMode::ZeroPageX });
    table[0x8C] = Some(Instruction { mnemonic: "STY", mode: AddressingMode::Absolute });

    // TAX
    table[0xAA] = Some(Instruction { mnemonic: "TAX", mode: AddressingMode::Implied });

    // TAY
    table[0xA8] = Some(Instruction { mnemonic: "TAY", mode: AddressingMode::Implied });

    // TSX
    table[0xBA] = Some(Instruction { mnemonic: "TSX", mode: AddressingMode::Implied });

    // TXA
    table[0x8A] = Some(Instruction { mnemonic: "TXA", mode: AddressingMode::Implied });

    // TXS
    table[0x9A] = Some(Instruction { mnemonic: "TXS", mode: AddressingMode::Implied });

    // TYA
    table[0x98] = Some(Instruction { mnemonic: "TYA", mode: AddressingMode::Implied });

    table
};

pub fn disassemble_bank(
    db: &DisassemblyInfo,
    bank_id: u8,
    rom_data: &[u8],
) -> Vec<DisassemblyLine> {
    let mut lines = Vec::new();
    let bank_info = match db.bank.get(&bank_id) {
        Some(info) => info,
        None => return lines,
    };

    let mut regions = bank_info.region.clone();
    regions.sort_by_key(|r| match r {
        RegionInfo::Code(range) => *range.start(),
        RegionInfo::Bytes(range) => *range.start(),
        RegionInfo::Words(range) => *range.start(),
    });

    // Detect base address from BankInfo::mapped_at or regions
    let mut base_address = bank_info.mapped_at.unwrap_or(0x8000);
    if bank_info.mapped_at.is_none() {
        if let Some(first_region) = regions.first() {
            let start = match first_region {
                RegionInfo::Code(r) => *r.start(),
                RegionInfo::Bytes(r) => *r.start(),
                RegionInfo::Words(r) => *r.start(),
            };
            let window_size = db.mapper_window_size as u16 * 1024;
            base_address = (start / window_size) * window_size;
        }
    }

    let mapper_size = db.mapper_window_size as u32 * 1024;
    let bank_start = base_address as u32;
    let bank_end = bank_start + mapper_size - 1;

    // First Pass: Collect all target addresses in ROM region ($8000-$FFFF)
    let mut targets = HashSet::new();
    for region in &regions {
        match region {
            RegionInfo::Code(range) => {
                let mut pc = *range.start() as u32;
                let end = *range.end() as u32;
                while pc <= end {
                    let offset = (pc.wrapping_sub(base_address as u32)) as usize;
                    if offset >= rom_data.len() { break; }
                    let opcode = rom_data[offset];
                    if let Some(instr) = &OPCODES[opcode as usize] {
                        let len = instr.mode.operand_length() as u32;
                        let mut op_val: u32 = 0;
                        for j in 1..=len {
                            if offset + (j as usize) < rom_data.len() {
                                op_val |= (rom_data[offset + (j as usize)] as u32) << (8 * (j - 1));
                            }
                        }
                        let (_, target_addr) = resolve_target(Some(instr.mode), op_val, pc as u16, db, bank_id);
                        if let Some(addr) = target_addr {
                            if addr >= 0x8000 { targets.insert(addr); }
                        }
                        pc += 1 + len;
                    } else {
                        pc += 1;
                    }
                }
            }
            RegionInfo::Words(range) => {
                let mut pc = *range.start() as u32;
                let end = *range.end() as u32;
                while pc <= end {
                    let offset = (pc.wrapping_sub(base_address as u32)) as usize;
                    if offset + 1 >= rom_data.len() { break; }
                    let low = rom_data[offset];
                    let high = rom_data[offset + 1];
                    let val = (high as u16) << 8 | (low as u16);
                    if val >= 0x8000 { targets.insert(val); }
                    pc += 2;
                }
            }
            _ => {}
        }
    }

    // Fill gaps with Bytes regions
    let mut filled_regions = Vec::new();
    let mut current_pc = bank_start;

    for region in regions {
        let (r_start, r_end) = match &region {
            RegionInfo::Code(r) => (*r.start() as u32, *r.end() as u32),
            RegionInfo::Bytes(r) => (*r.start() as u32, *r.end() as u32),
            RegionInfo::Words(r) => (*r.start() as u32, *r.end() as u32),
        };

        if r_start > current_pc {
            filled_regions.push(RegionInfo::Bytes((current_pc as u16)..=(r_start as u16 - 1)));
        }
        filled_regions.push(region);
        current_pc = r_end + 1;
    }

    if current_pc <= bank_end {
        filled_regions.push(RegionInfo::Bytes((current_pc as u16)..=(bank_end as u16)));
    }

    for region in filled_regions {
        match region {
            RegionInfo::Code(range) => {
                let mut pc = *range.start() as u32;
                let end = *range.end() as u32;
                while pc <= end {
                    let offset = (pc.wrapping_sub(base_address as u32)) as usize;
                    if offset >= rom_data.len() { break; }

                    let opcode = rom_data[offset];
                    let instr = &OPCODES[opcode as usize];
                    
                    let mut op_val: u32 = 0;
                    let (bytes, prefix, main, suffix, is_sym, mnemonic, length) = match instr {
                        Some(i) => {
                            let len = i.mode.operand_length() as u32;
                            let mut b = format!("{:02X}", opcode);
                            for j in 1..=len {
                                if offset + (j as usize) < rom_data.len() {
                                    let byte = rom_data[offset + (j as usize)];
                                    b.push_str(&format!(" {:02X}", byte));
                                    op_val |= (byte as u32) << (8 * (j - 1));
                                }
                            }
                            
                            let (p, m, s, sym) = format_operand(i.mode, op_val, pc as u16, db, bank_id, &targets);
                            (b, p, m, s, sym, i.mnemonic, 1 + len)
                        }
                        None => (format!("{:02X}", opcode), String::new(), String::new(), String::new(), false, "???", 1),
                    };

                    let annotation = get_annotation(db, bank_id, pc as u16);
                    let (target_bank, target_addr) = resolve_target(instr.as_ref().map(|i| i.mode), op_val, pc as u16, db, bank_id);

                    let mut line_symbol = annotation.symbol;
                    if line_symbol.is_none() && targets.contains(&(pc as u16)) {
                        line_symbol = Some(format!("L{:04X}", pc));
                    }

                    lines.push(DisassemblyLine {
                        address_label: format!("${:02X}:${:04X}", bank_id, pc),
                        address: pc as u16,
                        bank: bank_id as i16,
                        bytes,
                        opcode: mnemonic.to_string(),
                        operand_prefix: prefix,
                        operand_main: main,
                        operand_suffix: suffix,
                        operand_is_symbol: is_sym,
                        symbol: line_symbol,
                        comment: annotation.comment,
                        block_comment: annotation.block_comment,
                        target_bank,
                        target_address: target_addr,
                    });

                    pc += length;
                }
            }
            RegionInfo::Bytes(range) => {
                let mut pc = *range.start() as u32;
                let end = *range.end() as u32;
                while pc <= end {
                    let mut count = 0;
                    let mut bytes_str = String::new();
                    let mut hex_bytes = String::new();
                    let start_pc = pc;

                    while pc <= end && count < 8 {
                        if count > 0 && has_symbol(db, bank_id, pc as u16) {
                            break;
                        }
                        // Break on auto-labels too
                        if count > 0 && targets.contains(&(pc as u16)) {
                            break;
                        }

                        let offset = (pc.wrapping_sub(base_address as u32)) as usize;
                        if offset >= rom_data.len() { break; }

                        let val = rom_data[offset];
                        if !bytes_str.is_empty() { bytes_str.push_str(", "); }
                        bytes_str.push_str(&format!("${:02X}", val));
                        
                        if !hex_bytes.is_empty() { hex_bytes.push(' '); }
                        hex_bytes.push_str(&format!("{:02X}", val));

                        pc += 1;
                        count += 1;
                    }

                    if count > 0 {
                        let annotation = get_annotation(db, bank_id, start_pc as u16);
                        let mut line_symbol = annotation.symbol;
                        if line_symbol.is_none() && targets.contains(&(start_pc as u16)) {
                            line_symbol = Some(format!("L{:04X}", start_pc));
                        }

                        lines.push(DisassemblyLine {
                            address_label: format!("${:02X}:${:04X}", bank_id, start_pc),
                            address: start_pc as u16,
                            bank: bank_id as i16,
                            bytes: hex_bytes,
                            opcode: ".byt".to_string(),
                            operand_prefix: String::new(),
                            operand_main: bytes_str,
                            operand_suffix: String::new(),
                            operand_is_symbol: false,
                            symbol: line_symbol,
                            comment: annotation.comment,
                            block_comment: annotation.block_comment,
                            target_bank: None,
                            target_address: None,
                        });
                    } else {
                        break; 
                    }
                }
            }
            RegionInfo::Words(range) => {
                let mut pc = *range.start() as u32;
                let end = *range.end() as u32;
                while pc <= end {
                    let start_pc = pc;
                    let offset = (pc.wrapping_sub(base_address as u32)) as usize;
                    if offset + 1 >= rom_data.len() { break; }

                    let low = rom_data[offset];
                    let high = rom_data[offset + 1];
                    let val = (high as u16) << 8 | (low as u16);

                    let (main, is_sym) = resolve_symbol(val, db, bank_id, false, &targets);
                    let annotation = get_annotation(db, bank_id, start_pc as u16);
                    
                    let (target_bank, target_addr) = resolve_target(Some(AddressingMode::Absolute), val as u32, pc as u16, db, bank_id);

                    let mut line_symbol = annotation.symbol;
                    if line_symbol.is_none() && targets.contains(&(start_pc as u16)) {
                        line_symbol = Some(format!("L{:04X}", start_pc));
                    }

                    lines.push(DisassemblyLine {
                        address_label: format!("${:02X}:${:04X}", bank_id, start_pc),
                        address: start_pc as u16,
                        bank: bank_id as i16,
                        bytes: format!("{:02X} {:02X}", low, high),
                        opcode: ".word".to_string(),
                        operand_prefix: String::new(),
                        operand_main: main,
                        operand_suffix: String::new(),
                        operand_is_symbol: is_sym,
                        symbol: line_symbol,
                        comment: annotation.comment,
                        block_comment: annotation.block_comment,
                        target_bank,
                        target_address: target_addr,
                    });

                    pc += 2;
                }
            }
        }
    }

    lines
}

fn resolve_target(mode: Option<AddressingMode>, value: u32, pc: u16, db: &DisassemblyInfo, bank_id: u8) -> (Option<u8>, Option<u16>) {
    let mode = match mode {
        Some(m) => m,
        None => return (None, None),
    };

    let target_addr = match mode {
        AddressingMode::ZeroPage | AddressingMode::ZeroPageX | AddressingMode::ZeroPageY => Some(value as u16),
        AddressingMode::Absolute | AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => Some(value as u16),
        AddressingMode::Relative => {
            let offset = value as i8;
            Some(pc.wrapping_add(2).wrapping_add(offset as u16))
        }
        _ => None,
    };

    if let Some(addr) = target_addr {
        let current_fixed = db.bank.get(&bank_id).map(|b| b.is_fixed).unwrap_or(false);

        // 1. Check local bank
        if let Some(bank) = db.bank.get(&bank_id) {
            if bank.address.contains_key(&addr) {
                return (Some(bank_id), Some(addr));
            }
        }

        if current_fixed {
            // Resolution for fixed bank: Local -> Others -> Global
            for (other_id, other_bank) in &db.bank {
                if *other_id != bank_id && other_bank.address.contains_key(&addr) {
                    return (Some(*other_id), Some(addr));
                }
            }
        } else {
            // Resolution for non-fixed bank: Local -> Fixed -> Global
            for (other_id, other_bank) in &db.bank {
                if other_bank.is_fixed && other_bank.address.contains_key(&addr) {
                    return (Some(*other_id), Some(addr));
                }
            }
        }

        if db.global.contains_key(&addr) {
             return (None, Some(addr));
        }

        return (None, Some(addr));
    }

    (None, None)
}

fn has_symbol(db: &DisassemblyInfo, bank_id: u8, address: u16) -> bool {
    if let Some(bank) = db.bank.get(&bank_id) {
        if let Some(anno) = bank.address.get(&address) {
            return anno.symbol.is_some();
        }
    }
    if let Some(anno) = db.global.get(&address) {
        return anno.symbol.is_some();
    }
    false
}

fn get_annotation(db: &DisassemblyInfo, bank_id: u8, address: u16) -> AnnotationInfo {
    let mut result = AnnotationInfo::default();
    
    if let Some(anno) = db.global.get(&address) {
        result.symbol = anno.symbol.clone();
        result.comment = anno.comment.clone();
        result.block_comment = anno.block_comment.clone();
    }

    if let Some(bank) = db.bank.get(&bank_id) {
        if let Some(anno) = bank.address.get(&address) {
            if anno.symbol.is_some() { result.symbol = anno.symbol.clone(); }
            if anno.comment.is_some() { result.comment = anno.comment.clone(); }
            if anno.block_comment.is_some() { result.block_comment = anno.block_comment.clone(); }
        }
    }

    result
}

fn format_operand(mode: AddressingMode, value: u32, pc: u16, db: &DisassemblyInfo, bank_id: u8, targets: &HashSet<u16>) -> (String, String, String, bool) {
    match mode {
        AddressingMode::Implied => (String::new(), String::new(), String::new(), false),
        AddressingMode::Accumulator => (String::new(), "A".to_string(), String::new(), false),
        AddressingMode::Immediate => ("#".to_string(), format!("${:02X}", value), String::new(), false),
        AddressingMode::ZeroPage => {
            let (m, sym) = resolve_symbol(value as u16, db, bank_id, true, targets);
            (String::new(), m, String::new(), sym)
        }
        AddressingMode::ZeroPageX => {
            let (m, sym) = resolve_symbol(value as u16, db, bank_id, true, targets);
            (String::new(), m, ",X".to_string(), sym)
        }
        AddressingMode::ZeroPageY => {
            let (m, sym) = resolve_symbol(value as u16, db, bank_id, true, targets);
            (String::new(), m, ",Y".to_string(), sym)
        }
        AddressingMode::Relative => {
            let offset = value as i8;
            let target = pc.wrapping_add(2).wrapping_add(offset as u16);
            let (m, sym) = resolve_symbol(target, db, bank_id, false, targets);
            (String::new(), m, String::new(), sym)
        }
        AddressingMode::Absolute => {
            let (m, sym) = resolve_symbol(value as u16, db, bank_id, false, targets);
            (String::new(), m, String::new(), sym)
        }
        AddressingMode::AbsoluteX => {
            let (m, sym) = resolve_symbol(value as u16, db, bank_id, false, targets);
            (String::new(), m, ",X".to_string(), sym)
        }
        AddressingMode::AbsoluteY => {
            let (m, sym) = resolve_symbol(value as u16, db, bank_id, false, targets);
            (String::new(), m, ",Y".to_string(), sym)
        }
        AddressingMode::Indirect => {
            ( "(".to_string(), format!("${:04X}", value), ")".to_string(), false)
        }
        AddressingMode::IndexedIndirect => {
            ( "(".to_string(), format!("${:02X}", value), ",X)".to_string(), false)
        }
        AddressingMode::IndirectIndexed => {
            ( "(".to_string(), format!("${:02X}", value), "),Y".to_string(), false)
        }
    }
}

fn resolve_symbol(address: u16, db: &DisassemblyInfo, bank_id: u8, is_zp: bool, targets: &HashSet<u16>) -> (String, bool) {
    let current_fixed = db.bank.get(&bank_id).map(|b| b.is_fixed).unwrap_or(false);

    // Rule 1: Check local bank
    if let Some(bank) = db.bank.get(&bank_id) {
        if let Some(anno) = bank.address.get(&address) {
            if let Some(ref sym) = anno.symbol {
                return (sym.clone(), true);
            }
        }
    }

    if current_fixed {
        // Resolution for fixed bank: Local -> Other Banks -> Global
        for (other_id, other_bank) in &db.bank {
            if *other_id == bank_id { continue; }
            if let Some(anno) = other_bank.address.get(&address) {
                if let Some(ref sym) = anno.symbol {
                    return (sym.clone(), true);
                }
            }
        }
    } else {
        // Resolution for non-fixed bank: Local -> Fixed Banks -> Global
        for (_other_id, other_bank) in &db.bank {
            if !other_bank.is_fixed { continue; }
            if let Some(anno) = other_bank.address.get(&address) {
                if let Some(ref sym) = anno.symbol {
                    return (sym.clone(), true);
                }
            }
        }
    }

    // Check global address
    if let Some(anno) = db.global.get(&address) {
        if let Some(ref sym) = anno.symbol {
            return (sym.clone(), true);
        }
    }
    
    // Check for auto-label in ROM region
    if address >= 0x8000 && targets.contains(&address) {
        return (format!("L{:04X}", address), true);
    }

    if is_zp {
        (format!("${:02X}", address), false)
    } else {
        (format!("${:04X}", address), false)
    }
}
