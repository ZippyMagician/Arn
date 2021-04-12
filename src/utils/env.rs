use std::collections::HashMap;

use super::types::Dynamic;

pub struct Environment {
    pub funcs: HashMap<String, Box<dyn Fn(Dynamic) -> Dynamic>>,
    pub vars: HashMap<String, Dynamic>
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

    pub fn define_var<T>(&mut self, name: &str, val: T)
    where
        Dynamic: From<T>
    {
        self.vars.insert(name.to_owned(), Dynamic::from(val));
    }
}