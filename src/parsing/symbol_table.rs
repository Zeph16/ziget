use std::collections::HashMap;

use super::node::TypeNode;

#[derive(Debug, Clone)]
pub enum SymbolType {
    Variable(TypeNode),
    Procedure {
        return_type: TypeNode,
        param_types: Vec<TypeNode>,
    },
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub symbol_type: SymbolType,
    pub used: bool,
    pub initialized: bool,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub id: usize, 
    pub parent_id: Option<usize>,
    pub symbols: HashMap<String, SymbolInfo>,
    pub children: Vec<usize>,
}

impl SymbolTable {
    pub fn new(id: usize, parent_id: Option<usize>) -> Self {
        SymbolTable {
            id,
            parent_id,
            symbols: HashMap::new(),
            children: Vec::new(),
        }
    }

    pub fn insert(&mut self, name: String, symbol_info: SymbolInfo) {
        self.symbols.insert(name, symbol_info);
    }

    pub fn lookup<'a>(&'a self, name: &str, symbol_tables: &'a HashMap<usize, SymbolTable>) -> Option<&'a SymbolInfo> {
        if let Some(symbol_info) = self.symbols.get(name) {
            return Some(symbol_info);
        }

        if let Some(parent_id) = self.parent_id {
            if let Some(parent_table) = symbol_tables.get(&parent_id) {
                return parent_table.lookup(name, symbol_tables);
            }
        }

        None
    }

    pub fn check_unused_symbols(&self, warnings: &mut Vec<String>) {
        for (name, symbol) in &self.symbols {
            if !symbol.used {
                if let SymbolType::Variable(_) = symbol.symbol_type {
                    warnings.push(format!("Variable '{}' is declared but never used.", name));
                } else {
                    warnings.push(format!("Procedure '{}' is declared but never used.", name));
                }
            }
        }
    }
}
