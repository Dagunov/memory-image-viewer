#![windows_subsystem = "windows"]

use log::{error, info};
use read_process_memory::{copy_address, Pid, ProcessHandle};

mod app;
mod imageprocessing;

fn main() {
    env_logger::init();
    info!("Working in GUI mode");
    let native_options = eframe::NativeOptions::default();
    let clear_mem = std::env::args().len() > 1;
    if let Err(e) = eframe::run_native(
        "memory-image-viewer",
        native_options,
        Box::new(move |cc| Box::new(app::Application::new(cc, clear_mem))),
    ) {
        error!("Eframe init failed: {:?}", e);
    }
}

fn parse_address(address: &str) -> Result<usize, ()> {
    usize::from_str_radix(address.trim_start_matches("0x"), 16).map_err(|_| ())
}

fn get_bytes(pid: u32, address: usize, length: usize) -> std::io::Result<Vec<u8>> {
    let handle = ProcessHandle::try_from(pid as Pid)?;
    copy_address(address, length, &handle)
}
