use log::info;
use pretty_env_logger::env_logger::filter::Filter;
//use pretty_env_logger::formatted_builder;
use std::thread::Builder;

use crate::system::System;

mod cart;
mod cpu;
mod ir;
mod rcp;
mod rdram;
mod rsp;
mod system;

fn main() {
    pretty_env_logger::init();
    info!("program start");

    //let mut system = System::new();

    let emu_thread = Builder::new()
        .name("emu thread".to_string())
        .spawn(move || {
            let mut sys = System::new();
            sys.run()
        })
        .unwrap();

    emu_thread.join().unwrap().unwrap();
}
