use std::{cell::RefCell, rc::Rc, time::Instant};

use egui_notify::Toasts;
use serde::{Deserialize, Serialize};

use crate::{imageprocessing, parse_address};

mod app_ui;
mod image_view;

/// Configuration sets how to read an image
/// and where from.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Config {
    pid_label: String,
    pid: u32,
    address: String,
    width: u32,
    height: u32,
    data_type: imageprocessing::DataType,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pid_label: String::from("☰ Not selected!"),
            pid: 0,
            address: String::new(),
            width: 0,
            height: 0,
            data_type: Default::default(),
        }
    }
}
impl Config {
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

impl Default for SysInfo {
    fn default() -> Self {
        Self {
            system: sysinfo::System::new_with_specifics(
                sysinfo::RefreshKind::new().with_processes(sysinfo::ProcessRefreshKind::new()),
            ),
            last_update: Instant::now(),
            update_every: std::time::Duration::from_secs(1),
            search_filter: String::new(),
        }
    }
}

impl SysInfo {
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

impl Default for Toaster {
    fn default() -> Self {
        Self {
            toasts: Rc::new(RefCell::new(Toasts::default())),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Application {
    config: Config,
    #[serde(skip)]
    last_config: Option<Config>,
    #[serde(skip)]
    sysinfo: SysInfo,
    #[serde(skip)]
    toaster: Toaster,
    #[serde(skip)]
    image_view: Option<image_view::ImageView>,
    dump_folder: Option<std::path::PathBuf>,
}

impl Application {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or(Default::default())
    }
}

/// Checks if given process info passes
/// given filter. If it passes (or filter
/// is empty), this function returns true.
fn check_process_filter(pid: &str, pname: &str, filter: &str) -> bool {
    filter.is_empty()
        || (pid.contains(filter) || pname.to_lowercase().contains(&filter.to_lowercase()))
}
