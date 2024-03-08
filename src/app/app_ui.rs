use std::{cell::RefCell, rc::Rc};

use clap::ValueEnum;
use eframe::{
    egui::{
        text::{CCursor, CCursorRange},
        *,
    },
    App,
};
use egui_notify::Toasts;
use log::{error, info, warn};

use crate::{get_bytes, imageprocessing::DataType, parse_address};

use super::{check_process_filter, image_view::ImageView, Application, Toaster};

#[derive(Debug)]
enum GetImageError {
    AddressNotValid,
    MemoryReadError(std::io::Error),
}

impl Toaster {
    pub fn new() -> Self {
        Self {
            toasts: Rc::new(RefCell::new(Toasts::default())),
        }
    }

    fn draw(&self, ctx: &Context) {
        self.toasts.borrow_mut().show(ctx);
    }

    /// creates toasts info notification as well as prints to log
    pub fn info(&mut self, s: impl Into<String>, secs: u64) {
        let s = s.into();
        info!("{}", s);
        self.toasts
            .borrow_mut()
            .info(s)
            .set_duration(Some(std::time::Duration::from_secs(secs)));
    }

    /// creates toasts warn notification as well as prints to log
    pub fn warn(&mut self, s: impl Into<String>, secs: u64) {
        let s = s.into();
        warn!("{}", s);
        self.toasts
            .borrow_mut()
            .warning(s)
            .set_duration(Some(std::time::Duration::from_secs(secs)));
    }

    /// creates toasts error notification as well as prints to log
    pub fn error(&mut self, s: impl Into<String>) {
        let s = s.into();
        error!("{}", s);
        self.toasts
            .borrow_mut()
            .error(s)
            .set_duration(None)
            .set_closable(true);
    }
}

impl App for Application {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| self.draw_config(ui));

                ui.separator();

                ui.vertical(|ui| self.draw_image_block(ui));
            });
        });

        self.toaster.draw(ctx);
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
            if !self.config.address.is_empty() && !parse_address(&self.config.address).is_ok() {
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
        if self.config.is_filled() {
            if self.last_config.is_none()
                || self.last_config.as_ref().is_some_and(|c| &self.config != c)
            {
                match self.get_image(ui) {
                    Ok(_) => info!("Image loaded!"), // may spam, user will see if all ok
                    Err(e) => {
                        self.toaster.warn(format!("Image not loaded: {:?}", e), 2);
                    }
                }
                self.last_config = Some(self.config.clone());
            }
        }

        if let Some(image_view) = self.image_view.as_mut() {
            image_view.draw(ui);
        }

        // if self.saving_thread.is_some() {
        //     ui.spinner();
        // }
        // if ui
        //     .add_enabled(
        //         self.image.is_some() && self.saving_thread.is_none(),
        //         Button::new("Save"),
        //     )
        //     .clicked()
        // {
        //     let image_copy = self.image.as_ref().unwrap().clone();
        //     self.saving_thread = Some(std::thread::spawn(move || {
        //         image_copy.save(&PathBuf::from("out/out.png"))
        //     }));
        // }

        // if let Some(texture) = &self.texture {
        //     ui.add(Image::from_texture(texture).max_size(vec2(200f32, 200f32)));
        // }
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
                    );
                    self.image_view =
                        Some(ImageView::new(image_data, self.toaster.clone(), ui.ctx()));
                    Ok(())
                }
                Err(e) => Err(GetImageError::MemoryReadError(e)),
            }
        } else {
            Err(GetImageError::AddressNotValid)
        }
    }
}
