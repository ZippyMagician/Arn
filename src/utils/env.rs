use std::collections::HashMap;

use super::types::Dynamic;

pub struct Environment {
    funcs: HashMap<String, Box<dyn Fn(Dynamic) -> Dynamic>>,
    vars: HashMap<String, Dynamic>
}

impl Environment {
    pub fn init() -> Self {
        Self {
            funcs: HashMap::new(),
            vars: HashMap::new()
        }
    }

    pub fn define_fn<T: 'static>(&mut self, name: &str, f: T)
    where
        T: Fn(Dynamic) -> Dynamic
    {
        self.funcs.insert(name.to_owned(), Box::new(f));
    }

    pub fn define_var(&mut self, name: &str, val: Dynamic) {
        self.vars.insert(name.to_owned(), val);
    }
}