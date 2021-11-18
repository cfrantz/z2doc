use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::Range;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct BankAddress(Option<i16>, u16);

#[derive(Debug, Default)]
pub struct Symtab {
    highbank_range: Range<usize>,
    highbank: Option<i16>,
    table: RefCell<HashMap<BankAddress, String>>,
}

impl Symtab {
    pub fn set_highbank(&mut self, range: Range<usize>, bank: Option<i16>) {
        self.highbank_range = range;
        self.highbank = bank;
    }

    fn get_bank(&self, bank: Option<i16>, addr: u16) -> Option<i16> {
        let addr = addr as usize;
        if addr < 0x8000 {
            None
        } else if self.highbank_range.contains(&addr) {
            self.highbank
        } else if bank == self.highbank {
            Some(i16::MIN)
        } else {
            bank
        }
    }

    pub fn put(&self, bank: Option<i16>, addr: u16, sym: &str) {
        let mut table = self.table.borrow_mut();
        let bank = self.get_bank(bank, addr);
        match table.entry(BankAddress(bank, addr)) {
            Entry::Occupied(o) => {
                log::error!(
                    "Symtab already contains {:x?} = {}.  Not adding {:?}",
                    o.key(),
                    o.get(),
                    sym
                );
            }
            Entry::Vacant(v) => {
                v.insert(sym.to_string());
            }
        };
    }

    pub fn maybe_put(&self, bank: Option<i16>, addr: u16, sym: &str) {
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
                v.insert(sym.to_string());
            }
        };
    }

    pub fn get(&self, bank: Option<i16>, addr: u16) -> Option<String> {
        let table = self.table.borrow();
        let bank = self.get_bank(bank, addr);
        if bank == Some(i16::MIN) {
            let mut candidates = table
                .iter()
                .filter(|(BankAddress(_, a), _)| *a == addr)
                .collect::<Vec<_>>();
            candidates.sort();
            candidates.get(0).map(|c| match c {
                (BankAddress(None, _), sym) => sym.to_string(),
                (BankAddress(Some(b), _), sym) => format!("bank{}_{}", b, sym),
            })
        } else {
            table.get(&BankAddress(bank, addr)).map(|sym| match bank {
                None => sym.to_string(),
                Some(b) => format!("bank{}_{}", b, sym),
            })
        }
    }
}
