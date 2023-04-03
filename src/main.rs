use log::info;
use std::thread::Builder;

use crate::system::System;

mod cpu;
mod rdram;
mod system;
fn main() {
    info!("program start");

    let mut system = System::new();

    let emu_thread = Builder::new()
        .name("emu thread".to_string())
        .spawn(move || system.run())
        .unwrap();

    emu_thread.join().unwrap();
}
