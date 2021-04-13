use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use super::types::Dynamic;

#[derive(Clone)]
pub struct Environment {
    pub funcs: HashMap<String, Rc<dyn Fn(Dynamic) -> Dynamic>>,
    pub vars: HashMap<String, Dynamic>,
}

impl Environment {
    pub fn init() -> Self {
        Self {
            funcs: HashMap::new(),
            vars: HashMap::new(),
        }
    }

    pub fn define_fn<T: 'static>(&mut self, name: &str, f: T)
    where
        T: Fn(Dynamic) -> Dynamic,
    {
        self.funcs.insert(name.to_owned(), Rc::new(f));
    }

    pub fn define_var<T>(&mut self, name: &str, val: T)
    where
        Dynamic: From<T>,
    {
        self.vars.insert(name.to_owned(), Dynamic::from(val));
    }

    pub fn attempt_call(&mut self, name: &str, arg: Dynamic) -> Dynamic {
        let f = self
            .funcs
            .get_mut(name)
            .unwrap_or_else(|| panic!("Unrecognized function {}", name));

        f(arg)
    }
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Environment")
            .field("vars", &self.vars)
            .finish()
    }
}