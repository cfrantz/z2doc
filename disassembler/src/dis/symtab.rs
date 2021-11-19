use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::RangeInclusive;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct BankAddress(Option<i16>, u16);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Symbol {
    synthetic: bool,
    symbol: String,
}

#[derive(Debug)]
pub struct Symtab {
    highbank_range: RangeInclusive<usize>,
    highbank: Option<i16>,
    table: RefCell<HashMap<BankAddress, Symbol>>,
}

impl Default for Symtab {
    fn default() -> Self {
        Symtab {
            highbank_range: RangeInclusive::new(0xFFFF, 0),
            highbank: None,
            table: RefCell::default(),
        }
    }
}

impl Symtab {
    pub fn set_highbank(&mut self, range: RangeInclusive<usize>, bank: Option<i16>) {
        self.highbank_range = range;
        self.highbank = bank;
    }

    fn get_bank(&self, bank: Option<i16>, addr: u16) -> Option<i16> {
        let addr = addr as usize;
        if addr < 0x8000 || bank.is_none() {
            // Addresses below 0x8000 are RAM or hardware and aren't in a bank.
            None
        } else if self.highbank_range.contains(&addr) {
            // Most NES games have a highbank mapped all the time.
            self.highbank
        } else if bank == self.highbank {
            // If we're in the highbank but asked for an address not in the
            // highbank range, then search all banks.
            Some(i16::MIN)
        } else {
            bank
        }
    }

    fn name(bank: Option<i16>, sym: &str) -> String {
        match bank {
            None => sym.to_string(),
            Some(i16::MIN) => sym.to_string(),
            Some(b) => format!("bank{}_{}", b, sym),
        }
    }

    pub fn put(&self, bank: Option<i16>, addr: u16, sym: &str) {
        let mut table = self.table.borrow_mut();
        let bank = self.get_bank(bank, addr);
        if bank == Some(i16::MIN) {
            log::error!("Skipping put for {:x?}:{:x?} = {}", bank, addr, sym);
            return;
        }
        match table.entry(BankAddress(bank, addr)) {
            Entry::Occupied(o) => {
                log::error!(
                    "Symtab already contains {:x?} = {:?}.  Not adding {:?}",
                    o.key(),
                    o.get(),
                    sym
                );
            }
            Entry::Vacant(v) => {
                v.insert(Symbol {
                    synthetic: false,
                    symbol: Self::name(bank, sym),
                });
            }
        };
    }

    pub fn synthetic_put(&self, bank: Option<i16>, addr: u16, sym: &str) {
        let bank = self.get_bank(bank, addr);
        if addr < 0x8000 || bank == Some(i16::MIN) {
            return;
        }
        let mut table = self.table.borrow_mut();
        match table.entry(BankAddress(bank, addr)) {
            Entry::Occupied(_) => {
                // Do nothing.
            }
            Entry::Vacant(v) => {
                v.insert(Symbol {
                    synthetic: true,
                    symbol: Self::name(bank, sym),
                });
            }
        };
    }

    pub fn promote(&self, bank: Option<i16>, addr: u16, sym: Option<&str>) {
        if let Some(s) = sym {
            if s.contains("+") || s.contains("-") {
                return;
            }
        } else {
            return;
        }
        let mut table = self.table.borrow_mut();
        let bank = self.get_bank(bank, addr);
        if let Some(mut s) = table.get_mut(&BankAddress(bank, addr)) {
            s.synthetic = false;
        } else {
            log::warn!(
                "Cannot promote non-existant symbol at bank={:?} addr={:x?}",
                bank,
                addr
            );
        }
    }

    fn _get(&self, bank: Option<i16>, addr: u16, check_global: bool) -> Option<Symbol> {
        let table = self.table.borrow();
        let bank = self.get_bank(bank, addr);
        if bank == Some(i16::MIN) {
            let mut candidates = table
                .iter()
                .filter(|(BankAddress(_, a), _)| *a == addr)
                .collect::<Vec<_>>();
            candidates.sort();
            candidates.get(0).map(|c| c.1.clone())
        } else {
            let sym = table.get(&BankAddress(bank, addr));
            if check_global && (sym.is_none() || sym.unwrap().synthetic) {
                let global = table.get(&BankAddress(None, addr));
                global.or(sym).cloned()
            } else {
                sym.cloned()
            }
        }
    }

    pub fn get(&self, bank: Option<i16>, addr: u16) -> Option<String> {
        self._get(bank, addr, true).map(|s| s.symbol)
    }

    pub fn get_label(&self, bank: Option<i16>, addr: u16) -> Option<String> {
        self._get(bank, addr, false).map(|s| s.symbol)
    }

    pub fn get_offset(&self, bank: Option<i16>, addr: u16) -> Option<String> {
        let mut fallback = None;
        //let bank = self.get_bank(bank, addr);
        // Exact address match
        if let Some(symbol) = self._get(bank, addr, true) {
            if !symbol.synthetic {
                return Some(symbol.symbol);
            } else {
                fallback = Some(symbol);
            }
        }
        // Is addr one more than a known symbol?
        if let Some(mut symbol) = self._get(bank, addr.wrapping_sub(1), true) {
            if !symbol.synthetic {
                symbol.symbol.push_str("+1");
                return Some(symbol.symbol);
            }
        }
        // Is addr one less than a known symbol?
        if let Some(mut symbol) = self._get(bank, addr.wrapping_add(1), true) {
            if !symbol.synthetic {
                symbol.symbol.push_str("-1");
                return Some(symbol.symbol);
            }
        }
        fallback.map(|s| s.symbol)
    }

    pub fn get_globals(&self) -> Vec<(u16, String)> {
        let table = self.table.borrow();
        let mut globals = table
            .iter()
            .filter_map(|(BankAddress(bank, addr), s)| {
                if bank.is_none() {
                    Some((*addr, s.symbol.clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        globals.sort();
        globals
    }
}
