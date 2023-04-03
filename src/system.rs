use crate::cpu::Cpu;
use crate::rdram::Rdram;
use log::{debug, info};
use std::rc::Rc;

pub struct System {
    //every single hardware piece goes here
    //every piece of hardware gets an RF to every other piece of hardware that it can talk to
}

pub enum SystemResult {
    Graceful,
    Errored,
}

impl System {
    pub fn run(&mut self) -> Result<usize, SystemResult> {
        info!("system run start");

        debug!("constructing cpu");
        let ram = Rc::new(Rdram::default());

        let cpu = Rc::new(Cpu::new(ram.clone()));

        Ok(0)
    }

    pub fn new() -> Self {
        System {}
    }
}
