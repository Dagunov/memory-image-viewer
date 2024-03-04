use std::time::Instant;

use clap::ValueEnum;
use eframe::{egui, App};

use crate::{get_bytes, imageprocessing, parse_address};

const SCROLLVIEW_SIZE: egui::Vec2 = egui::vec2(500f32, 0f32);

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
}

impl App for Application {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_config(ui);

            if ui.button("Save").clicked() {
                let length = self.config.width as usize
                    * self.config.height as usize
                    * self.config.data_type.bytes_per_pixel() as usize;
                if let Ok(address) = parse_address(&self.config.address) {
                    let bytes = get_bytes(self.config.pid, address, length);
                    if !bytes.is_empty() {
                        let _ = imageprocessing::save_bytes(
                            self.config.data_type.convert_to_supported(bytes),
                            self.config.data_type,
                            "out.png",
                            [self.config.width, self.config.height],
                        );
                    }
                }
            }
        });
    }
}

impl Application {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            config: Config::new(),
            sysinfo: SysInfo::new(),
        }
    }

    fn draw_config(&mut self, ui: &mut eframe::egui::Ui) {
        let config = &mut self.config;
        let sysinfo = &mut self.sysinfo;

        // Process selection
        ui.horizontal(|ui| {
            ui.label("Process:");
            ui.menu_button(config.pid_label.clone(), |ui| {
                sysinfo.refresh();
                ui.add(
                    egui::TextEdit::singleline(&mut sysinfo.search_filter)
                        .hint_text("Search process")
                        .min_size(SCROLLVIEW_SIZE),
                )
                .request_focus();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (pid, process) in sysinfo.system.processes() {
                        let pid_string = pid.to_string();
                        if !check_process_filter(
                            &pid_string,
                            process.name(),
                            &sysinfo.search_filter,
                        ) {
                            continue;
                        }
                        let mut button = egui::Button::new(process.name())
                            .wrap(false)
                            .shortcut_text(&pid_string)
                            .min_size(SCROLLVIEW_SIZE);
                        if pid.as_u32() == config.pid {
                            button = button.selected(true);
                        }
                        if ui.add(button).clicked() {
                            config.pid_label = format!("☰ {}: {}", process.name(), pid_string);
                            config.pid = pid.as_u32();
                            ui.close_menu();
                        }
                    }
                });
            })
        });

        // Memory address input
        ui.add(egui::TextEdit::singleline(&mut config.address).hint_text("Memory address"));

        // Image size input
        egui::Grid::new("Sizes").show(ui, |ui| {
            ui.label("Width");
            ui.label("Height");
            ui.end_row();

            ui.add(egui::DragValue::new(&mut config.width));
            ui.add(egui::DragValue::new(&mut config.height));
        });

        // Data type selection
        egui::ComboBox::from_label("Data type")
            .selected_text(format!("{:?}", config.data_type))
            .show_ui(ui, |ui| {
                for val in imageprocessing::DataType::value_variants() {
                    ui.selectable_value(&mut config.data_type, *val, format!("{:?}", val));
                }
            });
    }
}

/// Checks if given process info passes
/// given filter. If it passes (or filter
/// is empty), this function returns true.
fn check_process_filter(pid: &str, pname: &str, filter: &str) -> bool {
    filter.is_empty()
        || (pid.contains(&filter) || pname.to_lowercase().contains(&filter.to_lowercase()))
}
