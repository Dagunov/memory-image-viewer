use clap::Parser;
use log::{error, trace, warn};
use read_process_memory::{copy_address, Pid, ProcessHandle};

mod app;
mod imageprocessing;

/// Tool which allows to save image from cv::Mat from memory
#[derive(Parser)]
#[command(about, version)]
struct CLI {
    /// PID of target process
    pid: u32,

    /// Target memory address in process
    addr: String,

    /// Width of image
    width: u32,

    /// Height of image
    height: u32,

    /// Buffer type
    #[arg(value_enum)]
    buf_type: imageprocessing::DataType,

    /// Out file name
    #[arg(short, long, default_value_t = String::from("out"))]
    out: String,
}

fn main() {
    env_logger::init();
    if std::env::args().len() > 1 {
        trace!("Working in CLI mode");
        let cli = CLI::parse();
        process_cli(cli);
    } else {
        trace!("Working in GUI mode");
        let native_options = eframe::NativeOptions::default();
        if let Err(e) = eframe::run_native(
            "MyApp",
            native_options,
            Box::new(|cc| Box::new(app::Application::new(cc))),
        ) {
            error!("Eframe init failed: {:?}", e);
        }
    }
}

fn process_cli(cli: CLI) {
    let buff_size =
        cli.width as usize * cli.height as usize * cli.buf_type.bytes_per_pixel() as usize;
    let addr = parse_address(&cli.addr).unwrap();
    let bytes = get_bytes(cli.pid, addr, buff_size);
    if !bytes.is_empty() {
        match imageprocessing::save_bytes(
            cli.buf_type.convert_to_supported(bytes),
            cli.buf_type,
            &(cli.out + ".png"),
            [cli.width, cli.height],
        ) {
            Ok(_) => println!("Image saved!"),
            Err(e) => println!("Could not save an image: {:?}", e),
        }
    }
}

fn parse_address(address: &str) -> Result<usize, ()> {
    usize::from_str_radix(address.trim_start_matches("0x"), 16).map_err(|_| ())
}

fn get_bytes(pid: u32, address: usize, length: usize) -> Vec<u8> {
    let handle = ProcessHandle::try_from(pid as Pid).unwrap();
    match copy_address(address, length, &handle) {
        Ok(bytes) => bytes,
        Err(e) => {
            warn!("Memory could not be read: {:?}", e);
            vec![]
        }
    }
}
