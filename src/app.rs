use std::{cell::RefCell, rc::Rc, time::Instant};

use egui_notify::Toasts;

use crate::{imageprocessing, parse_address};

mod app_ui;
mod image_view;

/// Configuration sets how to read an image
/// and where from.
#[derive(Clone, PartialEq, Eq)]
struct Config {
    pid_label: String,
    pid: u32,
    address: String,
    width: u32,
    height: u32,
    data_type: imageprocessing::DataType,
}

impl Config {
    fn new() -> Self {
        Self {
            pid_label: String::from("☰ Not selected!"),
            pid: 0,
            address: String::new(),
            width: 0,
            height: 0,
            data_type: imageprocessing::DataType::CV_8UC3,
        }
    }

    /// Checks if config is filled and all data is valid
    pub fn is_filled(&self) -> bool {
        self.pid != 0 && parse_address(&self.address).is_ok() && self.width != 0 && self.height != 0
    }
}

struct SysInfo {
    system: sysinfo::System,
    last_update: Instant,
    update_every: std::time::Duration,
    search_filter: String,
}

impl SysInfo {
    fn new() -> Self {
        Self {
            system: sysinfo::System::new_with_specifics(
                sysinfo::RefreshKind::new().with_processes(sysinfo::ProcessRefreshKind::new()),
            ),
            last_update: Instant::now(),
            update_every: std::time::Duration::from_secs(1),
            search_filter: String::new(),
        }
    }

    /// Refreshes info only if required
    fn refresh(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) > self.update_every {
            self.system
                .refresh_processes_specifics(sysinfo::ProcessRefreshKind::new());
            self.last_update = now;
        }
    }
}

#[derive(Clone)]
pub struct Toaster {
    toasts: Rc<RefCell<Toasts>>,
}

pub struct Application {
    config: Config,
    last_config: Option<Config>,
    sysinfo: SysInfo,
    toaster: Toaster,
    image_view: Option<image_view::ImageView>,
}

impl Application {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            config: Config::new(),
            last_config: None,
            sysinfo: SysInfo::new(),
            toaster: Toaster::new(),
            image_view: None,
        }
    }
}

/// Checks if given process info passes
/// given filter. If it passes (or filter
/// is empty), this function returns true.
fn check_process_filter(pid: &str, pname: &str, filter: &str) -> bool {
    filter.is_empty()
        || (pid.contains(filter) || pname.to_lowercase().contains(&filter.to_lowercase()))
}
