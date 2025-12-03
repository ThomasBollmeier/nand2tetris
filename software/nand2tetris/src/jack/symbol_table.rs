use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::jack::ast::{ClassVarCategory, Type};
use crate::vmtrans::ast::Segment;

pub struct SymbolTable {
    entries: HashMap<String, SymbolTableEntry>,
    next_index: HashMap<Segment, usize>,
    parent: Option<Rc<RefCell<SymbolTable>>>,
}

pub type SymbolTableRef = Rc<RefCell<SymbolTable>>;

#[derive(Debug, Clone)]
pub struct SymbolTableEntry {
    pub segment: Segment,
    pub index: u16,
    pub var_type: Type,
}

impl SymbolTable {
    pub fn new_ref(parent: Option<SymbolTableRef>) -> SymbolTableRef {
        Rc::new(RefCell::new(SymbolTable{
            entries: HashMap::new(),
            next_index: HashMap::new(),
            parent: parent.clone(),
        }))
    }

    pub fn get_parent(&mut self) -> Option<SymbolTableRef> {
        self.parent.clone()
    }

    pub fn get_entry(&self, name: &str) -> Option<SymbolTableEntry> {
        match self.entries.get(name) {
            Some(entry) => Some(entry.clone()),
            None => {
                if let Some(parent) = &self.parent {
                    parent.borrow().get_entry(name)
                } else {
                    None
                }
            }
        }
    }

    pub fn add_class_var(&mut self, name: String, category: ClassVarCategory, var_type: Type) {
        let segment = match category {
            ClassVarCategory::Static => Segment::Static,
            ClassVarCategory::Field => Segment::This,
        };
        let index = self.next_index_for_segment(segment);
        let entry = SymbolTableEntry {
            segment,
            index,
            var_type,
        };
        self.entries.insert(name, entry);
    }

    pub fn add_parameter(&mut self, name: String, var_type: Type) {
        self.add_var(Segment::Argument, name, var_type);
    }

    pub fn add_local(&mut self, name: String, var_type: Type) {
        self.add_var(Segment::Local, name, var_type);
    }

    pub fn add_var(&mut self, segment: Segment, name: String, var_type: Type) {
        let index = self.next_index_for_segment(segment);
        let entry = SymbolTableEntry {
            segment,
            index,
            var_type,
        };
        self.entries.insert(name, entry);
    }

    fn next_index_for_segment(&mut self, segment: Segment) -> u16 {
        let index = self.next_index.entry(segment).or_insert(0);
        let current_index = *index as u16;
        *index += 1;
        current_index
    }

}