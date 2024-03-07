use std::time::Instant;

use egui_notify::Toasts;

use crate::imageprocessing::{self, ImageData};

mod app_ui;

/// Configuration sets how to read an image
/// and where from.
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

pub struct Application {
    config: Config,
    sysinfo: SysInfo,
    image: Option<ImageData>,
    texture: Option<eframe::egui::TextureHandle>,
    toasts: Toasts,
}

impl Application {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            config: Config::new(),
            sysinfo: SysInfo::new(),
            image: None,
            texture: None,
            toasts: Toasts::default(),
        }
    }
}

/// Checks if given process info passes
/// given filter. If it passes (or filter
/// is empty), this function returns true.
fn check_process_filter(pid: &str, pname: &str, filter: &str) -> bool {
    filter.is_empty()
        || (pid.contains(&filter) || pname.to_lowercase().contains(&filter.to_lowercase()))
}
