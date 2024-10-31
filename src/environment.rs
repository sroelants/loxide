use std::collections::HashMap;

use crate::ast::LoxLiteral;

pub struct Env {
    table: HashMap<String, LoxLiteral>
}

impl Env {
    pub fn new() -> Self {
        Self { table: HashMap::new() }

    }
    pub fn define(&mut self, name: String, value: LoxLiteral) {
        self.table.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&LoxLiteral> {
        self.table.get(name)
    }
}
