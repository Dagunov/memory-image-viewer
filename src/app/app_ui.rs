use clap::ValueEnum;
use eframe::{
    egui::{
        text::{CCursor, CCursorRange},
        *,
    },
    App,
};
use log::{error, info};

use crate::{
    get_bytes,
    imageprocessing::{save_bytes, DataType},
    parse_address,
};

use super::{check_process_filter, Application};

impl Application {}

impl App for Application {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| self.draw_config(ui));

                ui.separator();

                ui.vertical(|ui| self.draw_image_block(ui));
            });
        });

        self.toasts.show(ctx);
    }
}

impl Application {
    /// creates toasts info notification as well as prints to log
    fn info(&mut self, s: impl Into<String>, secs: u64) {
        let s = s.into();
        info!("{}", s);
        self.toasts
            .info(s)
            .set_duration(Some(std::time::Duration::from_secs(secs)));
    }

    /// creates toasts error notification as well as prints to log
    fn error(&mut self, s: impl Into<String>) {
        let s = s.into();
        error!("{}", s);
        self.toasts.error(s).set_duration(None).set_closable(true);
    }

    fn draw_config(&mut self, ui: &mut eframe::egui::Ui) {
        ui.set_width(200f32);
        // Process selection
        let mut text_edit_output = None;
        if ui
            .menu_button(self.config.pid_label.clone(), |ui| {
                self.sysinfo.refresh();
                text_edit_output = Some(
                    TextEdit::singleline(&mut self.sysinfo.search_filter)
                        .hint_text("Search process")
                        .show(ui),
                );
                self.draw_processes(ui);
            })
            .response
            .clicked()
        {
            if let Some(mut teo) = text_edit_output.take() {
                let cc_range = CCursorRange::two(
                    CCursor::new(0),
                    CCursor::new(self.sysinfo.search_filter.len()),
                );
                teo.state.cursor.set_char_range(Some(cc_range));
                teo.state.store(ui.ctx(), teo.response.id);
                teo.response.request_focus();
            }
        }

        // Memory address input
        ui.add(TextEdit::singleline(&mut self.config.address).hint_text("Memory address"));

        // Image size input
        Grid::new("Sizes").show(ui, |ui| {
            ui.label("Width");
            ui.label("Height");
            ui.end_row();

            ui.add(DragValue::new(&mut self.config.width));
            ui.add(DragValue::new(&mut self.config.height));
        });

        // Data type selection
        ui.label("Data type");
        ComboBox::from_id_source("data_type_combobox")
            .selected_text(format!("{:?}", self.config.data_type))
            .show_ui(ui, |ui| {
                for val in DataType::value_variants() {
                    ui.selectable_value(&mut self.config.data_type, *val, format!("{:?}", val));
                }
            });
    }

    fn draw_processes(&mut self, ui: &mut Ui) {
        // This is a hack to draw processes better
        let longest_process_name_pid = {
            let mut pid_len = (sysinfo::Pid::from_u32(0), 0);
            for (pid, process) in self.sysinfo.system.processes() {
                if process.name().len() > pid_len.1 {
                    pid_len.0 = *pid;
                    pid_len.1 = process.name().len();
                }
            }
            pid_len.0
        };
        ScrollArea::vertical().show(ui, |ui| {
            let mut draw_single_process = |pid: &sysinfo::Pid, process: &sysinfo::Process| {
                let pid_string = pid.to_string();
                if !check_process_filter(&pid_string, process.name(), &self.sysinfo.search_filter) {
                    return;
                }
                let mut button = Button::new(process.name())
                    .wrap(false)
                    .shortcut_text(&pid_string);
                // .min_size(vec2(ui.available_width(), 0f32));
                if pid.as_u32() == self.config.pid {
                    button = button.selected(true);
                }
                if ui.add(button).clicked() {
                    self.config.pid_label = format!("☰ {}: {}", process.name(), pid_string);
                    self.config.pid = pid.as_u32();
                    ui.close_menu();
                }
            };
            if let Some(process) = self.sysinfo.system.process(longest_process_name_pid) {
                draw_single_process(&longest_process_name_pid, process);
            }
            for (pid, process) in self.sysinfo.system.processes() {
                if pid != &longest_process_name_pid {
                    draw_single_process(pid, process);
                }
            }
        });
    }

    fn draw_image_block(&mut self, ui: &mut Ui) {
        if ui.button("Get image").clicked() {
            let length = self.config.width as usize
                * self.config.height as usize
                * self.config.data_type.bytes_per_pixel() as usize;
            if let Ok(address) = parse_address(&self.config.address) {
                let bytes = get_bytes(self.config.pid, address, length);
                if !bytes.is_empty() {
                    self.info("Image loaded!", 2);
                    self.image = Some(self.config.data_type.init_image_data(
                        bytes,
                        self.config.width,
                        self.config.height,
                    ));
                    self.texture = None;
                } else {
                    self.toasts
                        .warning("Image was not loaded!")
                        .set_duration(Some(std::time::Duration::from_secs(2)));
                }
            }
        }

        if ui
            .add_enabled(self.image.is_some(), Button::new("Save"))
            .clicked()
        {
            match save_bytes(self.image.as_ref().unwrap(), "out.png") {
                Ok(_) => self.info("Image saved!", 3),
                Err(e) => self.error(format!("Image not saved: {}", e)),
            }
        }

        if let Some(image_data) = &self.image {
            let texture = self.texture.get_or_insert_with(|| {
                ui.ctx().load_texture(
                    "image",
                    image_data.init_egui_image(),
                    TextureOptions::NEAREST,
                )
            });
            ui.image(&*texture);
        }
    }
}
