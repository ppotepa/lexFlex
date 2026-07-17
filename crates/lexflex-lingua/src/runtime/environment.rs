use crate::id::SymbolId;
use crate::runtime::RuntimeValue;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct RuntimeEnvironment {
    locals: BTreeMap<SymbolId, RuntimeValue>,
}

impl RuntimeEnvironment {
    pub fn insert(&mut self, symbol: SymbolId, value: RuntimeValue) {
        self.locals.insert(symbol, value);
    }

    pub fn get(&self, symbol: &SymbolId) -> Option<&RuntimeValue> {
        self.locals.get(symbol)
    }

    pub fn len(&self) -> usize {
        self.locals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.locals.is_empty()
    }
}
