use std::collections::HashMap;

pub struct SymbolTable {
    entries: HashMap<String, SymbolTableEntry>
}

pub type Address = u16;

enum SymbolTableEntry {
    Label(Address),
    Symbol(Address),
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            entries: Self::create_entries()
        }
    }

    fn create_entries() -> HashMap<String, SymbolTableEntry> {
        let mut entries = HashMap::new();

        entries.insert("R0".to_string(), SymbolTableEntry::Symbol(0));
        entries.insert("R1".to_string(), SymbolTableEntry::Symbol(1));
        entries.insert("R2".to_string(), SymbolTableEntry::Symbol(2));
        entries.insert("R3".to_string(), SymbolTableEntry::Symbol(3));
        entries.insert("R4".to_string(), SymbolTableEntry::Symbol(4));
        entries.insert("R5".to_string(), SymbolTableEntry::Symbol(5));
        entries.insert("R6".to_string(), SymbolTableEntry::Symbol(6));
        entries.insert("R7".to_string(), SymbolTableEntry::Symbol(7));
        entries.insert("R8".to_string(), SymbolTableEntry::Symbol(8));
        entries.insert("R9".to_string(), SymbolTableEntry::Symbol(9));
        entries.insert("R10".to_string(), SymbolTableEntry::Symbol(10));
        entries.insert("R11".to_string(), SymbolTableEntry::Symbol(11));
        entries.insert("R12".to_string(), SymbolTableEntry::Symbol(12));
        entries.insert("R13".to_string(), SymbolTableEntry::Symbol(13));
        entries.insert("R14".to_string(), SymbolTableEntry::Symbol(14));
        entries.insert("R15".to_string(), SymbolTableEntry::Symbol(15));

        entries.insert("SP".to_string(), SymbolTableEntry::Symbol(0));
        entries.insert("LCL".to_string(), SymbolTableEntry::Symbol(1));
        entries.insert("ARG".to_string(), SymbolTableEntry::Symbol(2));
        entries.insert("THIS".to_string(), SymbolTableEntry::Symbol(3));
        entries.insert("THAT".to_string(), SymbolTableEntry::Symbol(4));

        entries.insert("SCREEN".to_string(), SymbolTableEntry::Symbol(16384));
        entries.insert("KBD".to_string(), SymbolTableEntry::Symbol(24576));

        entries
    }

    pub fn add_label(&mut self, name: &str, address: Address) {
        self.entries.insert(name.to_string(), SymbolTableEntry::Label(address));
    }

    pub fn add_symbol(&mut self, name: &str, address: Address) {
        self.entries.insert(name.to_string(), SymbolTableEntry::Symbol(address));
    }

    pub fn lookup(&self, name: &str) -> Option<Address> {
        self.entries.get(name).map(|entry| match entry {
            SymbolTableEntry::Label(address) => *address,
            SymbolTableEntry::Symbol(address) => *address,
        })
    }
}