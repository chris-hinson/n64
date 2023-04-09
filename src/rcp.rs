use crate::rsp::Rsp;
use std::cell::RefCell;
use std::rc::Rc;
//reality coprocessor
#[derive(Default)]
pub struct Rcp {
    pub rsp: Rc<RefCell<Rsp>>,
}

impl Rcp {
    pub fn new() -> Self {
        Self {
            rsp: Rc::new(RefCell::new(Rsp::default())),
        }
    }
}
