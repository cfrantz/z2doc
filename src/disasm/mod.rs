use crate::models::{AnnotationInfo, DisassemblyInfo, DisassemblyLine, RegionInfo};

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

    let mapper_size = db.mapper_window_size as usize * 1024;
    // Simple mapping: bank 0 maps to $8000, bank 1 to $C000 (for 16K)
    // For 8K: 0->$8000, 1->$A000, 2->$C000, 3->$E000
    let base_address = match mapper_size {
        8192 => 0x8000 + (bank_id as u16 % 4) * 0x2000,
        16384 => if bank_id % 2 == 0 { 0x8000 } else { 0xC000 },
        _ => 0x8000,
    };

    let mut regions = bank_info.region.clone();
    regions.sort_by_key(|r| match r {
        RegionInfo::Code(range) => *range.start(),
        RegionInfo::Bytes(range) => *range.start(),
        RegionInfo::Words(range) => *range.start(),
    });

    for region in regions {
        match region {
            RegionInfo::Code(range) => {
                let mut pc = *range.start();
                while pc <= *range.end() {
                    let offset = (pc - base_address) as usize;
                    if offset >= rom_data.len() { break; }

                    let opcode = rom_data[offset];
                    let instr = &OPCODES[opcode as usize];
                    
                    let (bytes, mnemonic, operand, length) = match instr {
                        Some(i) => {
                            let len = i.mode.operand_length();
                            let mut b = format!("{:02X}", opcode);
                            let mut op_val: u32 = 0;
                            for j in 1..=len {
                                if offset + j as usize < rom_data.len() {
                                    let byte = rom_data[offset + j as usize];
                                    b.push_str(&format!(" {:02X}", byte));
                                    op_val |= (byte as u32) << (8 * (j - 1));
                                }
                            }
                            
                            let op_str = format_operand(i.mode, op_val, pc, db, bank_id);
                            (b, i.mnemonic, op_str, 1 + len)
                        }
                        None => (format!("{:02X}", opcode), "???", String::new(), 1),
                    };

                    let annotation = get_annotation(db, bank_id, pc);
                    lines.push(DisassemblyLine {
                        address_label: format!("${:02X}:${:04X}", bank_id, pc),
                        address: pc,
                        bank: Some(bank_id),
                        bytes,
                        opcode: mnemonic.to_string(),
                        operand,
                        symbol: annotation.symbol,
                        comment: annotation.comment,
                        block_comment: annotation.block_comment,
                        is_target: false,
                    });

                    pc += length;
                }
            }
            RegionInfo::Bytes(range) => {
                let mut pc = *range.start();
                while pc <= *range.end() {
                    let mut count = 0;
                    let mut bytes_str = String::new();
                    let mut hex_bytes = String::new();
                    let start_pc = pc;

                    while pc <= *range.end() && count < 8 {
                        if count > 0 && has_symbol(db, bank_id, pc) {
                            break;
                        }

                        let offset = (pc - base_address) as usize;
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
                        let annotation = get_annotation(db, bank_id, start_pc);
                        lines.push(DisassemblyLine {
                            address_label: format!("${:02X}:${:04X}", bank_id, start_pc),
                            address: start_pc,
                            bank: Some(bank_id),
                            bytes: hex_bytes,
                            opcode: ".byt".to_string(),
                            operand: bytes_str,
                            symbol: annotation.symbol,
                            comment: annotation.comment,
                            block_comment: annotation.block_comment,
                            is_target: false,
                        });
                    } else {
                        break; 
                    }
                }
            }
            RegionInfo::Words(range) => {
                let mut pc = *range.start();
                while pc <= *range.end() {
                    let mut count = 0;
                    let mut words_str = String::new();
                    let mut hex_bytes = String::new();
                    let start_pc = pc;

                    while pc <= *range.end() && count < 4 {
                        if count > 0 && has_symbol(db, bank_id, pc) {
                            break;
                        }

                        let offset = (pc - base_address) as usize;
                        if offset + 1 >= rom_data.len() { break; }

                        let low = rom_data[offset];
                        let high = rom_data[offset + 1];
                        let val = (high as u16) << 8 | (low as u16);

                        if !words_str.is_empty() { words_str.push_str(", "); }
                        words_str.push_str(&format!("${:04X}", val));

                        if !hex_bytes.is_empty() { hex_bytes.push(' '); }
                        hex_bytes.push_str(&format!("{:02X} {:02X}", low, high));

                        pc += 2;
                        count += 1;
                    }

                    if count > 0 {
                        let annotation = get_annotation(db, bank_id, start_pc);
                        lines.push(DisassemblyLine {
                            address_label: format!("${:02X}:${:04X}", bank_id, start_pc),
                            address: start_pc,
                            bank: Some(bank_id),
                            bytes: hex_bytes,
                            opcode: ".word".to_string(),
                            operand: words_str,
                            symbol: annotation.symbol,
                            comment: annotation.comment,
                            block_comment: annotation.block_comment,
                            is_target: false,
                        });
                    } else {
                        break;
                    }
                }
            }
        }
    }

    lines
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

fn format_operand(mode: AddressingMode, value: u32, pc: u16, db: &DisassemblyInfo, bank_id: u8) -> String {
    match mode {
        AddressingMode::Implied => String::new(),
        AddressingMode::Accumulator => "A".to_string(),
        AddressingMode::Immediate => format!("#${:02X}", value),
        AddressingMode::ZeroPage => resolve_symbol(value as u16, db, bank_id, true),
        AddressingMode::ZeroPageX => format!("{},X", resolve_symbol(value as u16, db, bank_id, true)),
        AddressingMode::ZeroPageY => format!("{},Y", resolve_symbol(value as u16, db, bank_id, true)),
        AddressingMode::Relative => {
            let offset = value as i8;
            let target = pc.wrapping_add(2).wrapping_add(offset as u16);
            resolve_symbol(target, db, bank_id, false)
        }
        AddressingMode::Absolute => resolve_symbol(value as u16, db, bank_id, false),
        AddressingMode::AbsoluteX => format!("{},X", resolve_symbol(value as u16, db, bank_id, false)),
        AddressingMode::AbsoluteY => format!("{},Y", resolve_symbol(value as u16, db, bank_id, false)),
        AddressingMode::Indirect => format!("(${:04X})", value),
        AddressingMode::IndexedIndirect => format!("(${:02X},X)", value),
        AddressingMode::IndirectIndexed => format!("(${:02X}),Y", value),
    }
}

fn resolve_symbol(address: u16, db: &DisassemblyInfo, bank_id: u8, is_zp: bool) -> String {
    if let Some(bank) = db.bank.get(&bank_id) {
        if let Some(anno) = bank.address.get(&address) {
            if let Some(ref sym) = anno.symbol {
                return sym.clone();
            }
        }
    }
    if let Some(anno) = db.global.get(&address) {
        if let Some(ref sym) = anno.symbol {
            return sym.clone();
        }
    }
    
    if is_zp {
        format!("${:02X}", address)
    } else {
        format!("${:04X}", address)
    }
}
