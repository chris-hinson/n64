use log::info;
use std::thread::Builder;

use crate::system::System;

mod cart;
mod cpu;
mod rcp;
mod rdram;
mod rsp;
mod system;
fn main() {
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
