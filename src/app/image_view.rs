use std::{cell::RefCell, path::PathBuf, rc::Rc};

use eframe::egui::*;

use crate::imageprocessing::{ImageData, ImageProcessingResult};

use super::{file_helper::*, logger::Logger};

pub struct ImageView {
    image: ImageData,
    texture: TextureHandle,
    scale: f32,
    saving_thread: Option<std::thread::JoinHandle<(ImageProcessingResult, PathBuf)>>,
    logger: Rc<RefCell<Logger>>,
    /// for "Dump with name"
    dump_filename: Option<String>,
}

impl ImageView {
    pub fn new(image: ImageData, logger: Rc<RefCell<Logger>>, ctx: &Context) -> Self {
        let texture = ctx.load_texture("image", image.init_egui_image(), TextureOptions::NEAREST);
        Self {
            image,
            texture,
            scale: 1f32,
            saving_thread: None,
            logger,
            dump_filename: None,
        }
    }

    pub fn draw(&mut self, ui: &mut Ui, dump_folder: &mut Option<PathBuf>) {
        if self.saving_thread.as_ref().is_some_and(|t| t.is_finished()) {
            let t = self.saving_thread.take().unwrap();
            match t.join() {
                Ok((Ok(_), fname)) => self
                    .logger
                    .borrow_mut()
                    .info(format!("Image saved at {:?}", fname)),
                Ok((Err(e), _)) => self
                    .logger
                    .borrow_mut()
                    .error(format!("Image not saved: {:?}", e)),
                Err(e) => self
                    .logger
                    .borrow_mut()
                    .error(format!("Saving thread failed: {:?}", e)),
            };
        }

        ui.horizontal(|ui| {
            if self.saving_thread.is_some() {
                ui.spinner();
            }

            if ui.button("⊟").clicked() {
                self.scale *= 0.9f32;
            }
            if ui
                .button(format!("{}%", (self.scale * 100f32) as usize))
                .clicked()
            {
                self.scale = 1f32;
            }
            if ui.button("⊞").clicked() {
                self.scale *= 1.1f32;
                self.scale = self.scale.min(1f32);
            }
        });

        ui.horizontal(|ui| {
            if let Some(dump_filename) = &mut self.dump_filename {
                ui.text_edit_singleline(dump_filename).request_focus();
                if ui.ctx().input(|i| i.key_pressed(Key::Enter)) || ui.button("Save").clicked() {
                    assert!(dump_folder.is_some());
                    let dump_filename = dump_filename.clone();
                    self.save_to(dump_folder.as_ref().unwrap().join(dump_filename + ".png"));
                    self.dump_filename = None;
                }
            } else {
                ui.add_enabled_ui(self.saving_thread.is_none(), |ui| {
                    if ui.button("Save").clicked() {
                        if let Ok(new_file_path) =
                            file_dialog(FileDialogMode::SaveFile, &self.logger)
                        {
                            self.save_to(new_file_path);
                        }
                    }

                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        if ui.button("Dump with name").clicked() {
                            if dump_folder.is_some() {
                                self.dump_filename = Some(String::new());
                            } else {
                                if let Ok(path) =
                                    file_dialog(FileDialogMode::SelectFolder, &self.logger)
                                {
                                    dump_folder.replace(path);
                                }
                            }
                        }
                        if ui.button("Dump").clicked() {
                            if dump_folder.is_some() {
                                self.save_to(Self::gen_dump_path(dump_folder.clone().unwrap()));
                            } else {
                                if let Ok(path) =
                                    file_dialog(FileDialogMode::SelectFolder, &self.logger)
                                {
                                    dump_folder.replace(path);
                                }
                            }
                        }
                    });
                });
            }
        });

        ui.add(Image::from_texture(&self.texture).max_size(vec2(
            self.image.width as f32 * self.scale,
            self.image.height as f32 * self.scale,
        )));
    }

    fn save_to(&mut self, path: PathBuf) {
        let image_copy = self.image.clone();
        self.saving_thread = Some(std::thread::spawn(move || {
            (image_copy.save(&path), path.to_path_buf())
        }));
    }

    fn gen_dump_path(dump_folder: PathBuf) -> PathBuf {
        let now = chrono::Local::now();
        // todo: allow custom format
        let fname = now.format("%d%m%y_%H_%M_%S.png").to_string();
        let mut fpath = dump_folder.join(fname);
        let mut i = 1;
        while fpath.exists() {
            fpath = dump_folder.join(
                now.format(&format!("%d%m%y_%H_%M_%S({}).png", i))
                    .to_string(),
            );
            i += 1;
        }

        fpath
    }
}
