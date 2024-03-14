use std::fmt::Debug;

use clap::ValueEnum;
use eframe::{
    egui::{
        text::{CCursor, CCursorRange},
        *,
    },
    App,
};

use crate::{get_bytes, imageprocessing::*, parse_address};

use super::{check_process_filter, file_helper::*, image_view::ImageView, Application};

#[derive(Debug)]
enum GetImageError {
    AddressNotValid,
    MemoryReadError(std::io::Error),
}

impl App for Application {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            if self.in_settings {
                self.draw_settings(ui);
                if ui.input(|i| i.key_pressed(Key::Escape)) {
                    self.in_settings = false;
                }
            } else {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| self.draw_config(ui));

                    ui.separator();

                    ui.vertical(|ui| self.draw_image_block(ui));
                });
            }
        });

        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.logger.borrow().show_label(ui);
                ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                    if ui.button("⛭").clicked() {
                        self.in_settings = !self.in_settings;
                    }
                });
            });
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

impl Application {
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
        ui.vertical(|ui| {
            if !self.config.address.is_empty() && parse_address(&self.config.address).is_err() {
                ui.style_mut().visuals.override_text_color =
                    Some(ui.style().visuals.error_fg_color);
            }
            ui.add(TextEdit::singleline(&mut self.config.address).hint_text("Memory address"));
        });

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
                Grid::new("data_type_grid").show(ui, |ui| {
                    ui.selectable_value(
                        &mut self.config.data_type,
                        DataType::CV_8UC1,
                        format!("{:?}", DataType::CV_8UC1),
                    );
                    ui.selectable_value(
                        &mut self.config.data_type,
                        DataType::CV_16UC1,
                        format!("{:?}", DataType::CV_16UC1),
                    );
                    ui.selectable_value(
                        &mut self.config.data_type,
                        DataType::CV_32FC1,
                        format!("{:?}", DataType::CV_32FC1),
                    );
                    ui.selectable_value(
                        &mut self.config.data_type,
                        DataType::CV_64FC1,
                        format!("{:?}", DataType::CV_64FC1),
                    );
                    ui.end_row();

                    ui.selectable_value(
                        &mut self.config.data_type,
                        DataType::CV_8UC3,
                        format!("{:?}", DataType::CV_8UC3),
                    );
                    ui.selectable_value(
                        &mut self.config.data_type,
                        DataType::CV_16UC3,
                        format!("{:?}", DataType::CV_16UC3),
                    );
                    ui.selectable_value(
                        &mut self.config.data_type,
                        DataType::CV_32FC3,
                        format!("{:?}", DataType::CV_32FC3),
                    );
                    ui.selectable_value(
                        &mut self.config.data_type,
                        DataType::CV_64FC3,
                        format!("{:?}", DataType::CV_64FC3),
                    );
                    ui.end_row();

                    ui.selectable_value(
                        &mut self.config.data_type,
                        DataType::CV_8UC4,
                        format!("{:?}", DataType::CV_8UC4),
                    );
                    ui.selectable_value(
                        &mut self.config.data_type,
                        DataType::CV_16UC4,
                        format!("{:?}", DataType::CV_16UC4),
                    );
                    ui.selectable_value(
                        &mut self.config.data_type,
                        DataType::CV_32FC4,
                        format!("{:?}", DataType::CV_32FC4),
                    );
                    ui.selectable_value(
                        &mut self.config.data_type,
                        DataType::CV_64FC4,
                        format!("{:?}", DataType::CV_64FC4),
                    );
                    ui.end_row();
                });
            });

        // Channel order selection
        ui.add_enabled_ui(
            self.config.data_type.channels() == 3 || self.config.data_type.channels() == 4,
            |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(self.config.channel_order == ChannelOrder::Rgb, "RGB")
                        .clicked()
                    {
                        self.config.channel_order = ChannelOrder::Rgb;
                    }
                    if ui
                        .selectable_label(self.config.channel_order == ChannelOrder::Bgr, "BGR")
                        .clicked()
                    {
                        self.config.channel_order = ChannelOrder::Bgr;
                    }
                });
            },
        );
    }

    fn draw_processes(&mut self, ui: &mut Ui) {
        // This is a hack to draw processes better
        let longest_process_name_pid = {
            let mut pid_len = (sysinfo::Pid::from_u32(0), 0);
            for (pid, process) in self.sysinfo.system.processes() {
                if process.name().len() > pid_len.1
                    && check_process_filter(
                        &pid.to_string(),
                        process.name(),
                        &self.sysinfo.search_filter,
                    )
                {
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
        if self.config.is_filled()
            && (self.last_config.is_none()
                || self.last_config.as_ref().is_some_and(|c| &self.config != c))
        {
            self.sysinfo.refresh();
            if self
                .sysinfo
                .system
                .process(sysinfo::Pid::from_u32(self.config.pid))
                .is_none()
            {
                self.config.pid = 0;
                self.config.pid_label = "☰ Not selected!".into();
            }
            match self.get_image(ui) {
                Ok(_) => self.logger.borrow_mut().info("Image loaded!"),
                Err(e) => {
                    self.logger
                        .borrow_mut()
                        .warn(format!("Image not loaded: {:?}", e));
                }
            }
            self.last_config = Some(self.config.clone());
        }

        if let Some(image_view) = self.image_view.as_mut() {
            image_view.draw(ui, &mut self.dump_folder);
        }
    }

    fn get_image(&mut self, ui: &mut Ui) -> Result<(), GetImageError> {
        let length = self.config.width as usize
            * self.config.height as usize
            * self.config.data_type.bytes_per_pixel() as usize;
        if let Ok(address) = parse_address(&self.config.address) {
            match get_bytes(self.config.pid, address, length) {
                Ok(bytes) => {
                    let image_data = self.config.data_type.init_image_data(
                        bytes,
                        self.config.width,
                        self.config.height,
                        self.config.channel_order,
                    );
                    self.image_view =
                        Some(ImageView::new(image_data, self.logger.clone(), ui.ctx()));
                    Ok(())
                }
                Err(e) => Err(GetImageError::MemoryReadError(e)),
            }
        } else {
            Err(GetImageError::AddressNotValid)
        }
    }

    fn draw_settings(&mut self, ui: &mut Ui) {
        ui.label("Dump folder:");
        ui.horizontal(|ui| {
            if self.file_dialog_not_implemented {
                if self.dump_folder_text_edit.is_empty() && self.dump_folder.is_some() {
                    if let Ok(string) = self
                        .dump_folder
                        .clone()
                        .unwrap()
                        .into_os_string()
                        .into_string()
                    {
                        self.dump_folder_text_edit = string;
                    }
                }
                ui.text_edit_singleline(&mut self.dump_folder_text_edit);
                if ui.button("✅").clicked() || ui.input(|i| i.key_pressed(Key::Enter)) {
                    let path = std::path::PathBuf::from(self.dump_folder_text_edit.clone());
                    if path.exists() {
                        self.dump_folder = Some(path);
                    } else {
                        if self.dump_folder.is_some() {
                            if let Ok(string) = self
                                .dump_folder
                                .clone()
                                .unwrap()
                                .into_os_string()
                                .into_string()
                            {
                                self.dump_folder_text_edit = string;
                            }
                        } else {
                            self.dump_folder_text_edit.clear();
                        }
                    }
                }
            } else {
                ui.label(
                    self.dump_folder
                        .as_ref()
                        .map(|p| match p.clone().into_os_string().into_string() {
                            Ok(string) => string,
                            Err(e) => {
                                self.logger
                                    .borrow_mut()
                                    .error(format!("Failed to convert path to string: {:?}", e));
                                "Not set".into()
                            }
                        })
                        .unwrap_or("Not set".into()),
                );

                if ui.button("✏").clicked() {
                    match file_dialog(FileDialogMode::SelectFolder, &self.logger) {
                        Ok(path) => self.dump_folder = Some(path),
                        Err(FileDialogError::NoImplementation) => {
                            self.file_dialog_not_implemented = true;
                        }
                        _ => {}
                    }
                }
            }

            if ui.button("❌").clicked() {
                self.dump_folder = None;
                self.dump_folder_text_edit.clear();
            }
        });
    }
}
