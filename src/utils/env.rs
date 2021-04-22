use std::fmt;
use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

use super::types::{Dynamic, Env};

#[derive(Clone)]
pub struct Environment {
    pub vals: HashMap<String, Rc<dyn Fn(Env, Dynamic) -> Dynamic>>,
}

impl Environment {
    pub fn init() -> Self {
        Self {
            vals: HashMap::new(),
        }
    }

    pub fn define<T: 'static>(&mut self, name: &str, f: T)
    where
        T: Fn(Env, Dynamic) -> Dynamic,
    {
        self.vals.insert(name.trim().to_owned(), Rc::new(f));
    }

    pub fn define_var<T: 'static + Clone>(&mut self, name: &str, val: T)
    where
        Dynamic: From<T>,
    {
        self.vals.insert(
            name.trim().to_owned(),
            Rc::new(move |_, _| Dynamic::from(val.clone())),
        );
    }

    #[inline]
    pub fn get_var(&self, name: &str) -> Dynamic {
        // Dummy call, assumes it is a constant value
        self.attempt_call(
            name,
            &Rc::new(RefCell::new(Environment::init())),
            Dynamic::from(false),
        )
    }

    pub fn attempt_call(&self, name: &str, env: &Env, arg: Dynamic) -> Dynamic {
        let f = self
            .vals
            .get(name)
            .unwrap_or_else(|| panic!("Unrecognized value {}", name));
        f(Rc::clone(env), arg)
    }
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Environment").finish()
    }
}
