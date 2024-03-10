use std::path::PathBuf;

use eframe::egui::*;

use crate::imageprocessing::{ImageData, ImageProcessingResult};

use super::Toaster;

enum FileDialogMode {
    SaveFile,
    SelectFolder,
}

pub struct ImageView {
    image: ImageData,
    texture: TextureHandle,
    scale: f32,
    saving_thread: Option<std::thread::JoinHandle<ImageProcessingResult>>,
    toaster: Toaster,
    /// for "Dump with name"
    dump_filename: Option<String>,
}

impl ImageView {
    pub fn new(image: ImageData, toaster: Toaster, ctx: &Context) -> Self {
        let texture = ctx.load_texture("image", image.init_egui_image(), TextureOptions::NEAREST);
        Self {
            image,
            texture,
            scale: 1f32,
            saving_thread: None,
            toaster,
            dump_filename: None,
        }
    }

    pub fn draw(&mut self, ui: &mut Ui, dump_folder: &mut Option<PathBuf>) {
        if self.saving_thread.as_ref().is_some_and(|t| t.is_finished()) {
            let t = self.saving_thread.take().unwrap();
            match t.join() {
                Ok(_) => self.toaster.info("Image saved!", 3),
                Err(e) => self.toaster.error(format!("Image not saved: {:?}", e)),
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
                        if let Ok(new_file_path) = self.file_dialog(FileDialogMode::SaveFile) {
                            self.save_to(new_file_path);
                        }
                    }

                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        if ui.button("Dump with name").clicked() {
                            if dump_folder.is_some() {
                                self.dump_filename = Some(String::new());
                            } else {
                                if let Ok(path) = self.file_dialog(FileDialogMode::SelectFolder) {
                                    dump_folder.replace(path);
                                }
                            }
                        }
                        if ui.button("Dump").clicked() {
                            if dump_folder.is_some() {
                                self.save_to(Self::gen_dump_path(dump_folder.clone().unwrap()));
                            } else {
                                if let Ok(path) = self.file_dialog(FileDialogMode::SelectFolder) {
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
        self.saving_thread = Some(std::thread::spawn(move || image_copy.save(&path)));
    }

    fn file_dialog(&mut self, fd_mode: FileDialogMode) -> Result<PathBuf, ()> {
        let fd = native_dialog::FileDialog::new();
        let fd_res = match fd_mode {
            FileDialogMode::SaveFile => fd.add_filter("png", &["png"]).show_save_single_file(),
            FileDialogMode::SelectFolder => fd.show_open_single_dir(),
        };
        match fd_res {
            Ok(path) => {
                if let Some(path) = path {
                    Ok(path)
                } else {
                    self.toaster.info("Out location not selected", 3);
                    Err(())
                }
            }
            Err(native_dialog::Error::NoImplementation) => match fd_mode {
                FileDialogMode::SaveFile => {
                    self.toaster
                        .error("File dialog not supported on your OS, save file via \"Dump...\"");
                    Err(())
                }
                FileDialogMode::SelectFolder => {
                    self.toaster.error(
                        "File dialog not supported on your OS, enter path manually in settings",
                    );
                    Err(())
                }
            },
            Err(e) => {
                self.toaster
                    .error("File dialog error, enter path manually in settings or try again");
                self.toaster.error(format!("FileDialog error: {:?}", e));
                Err(())
            }
        }
    }

    fn gen_dump_path(dump_folder: PathBuf) -> PathBuf {
        let now = chrono::Local::now();
        // todo: allow custom format
        let fname = now.format("%d_%m__%H_%M_%S.png").to_string();
        let mut fpath = dump_folder.join(fname);
        let mut i = 1;
        while fpath.exists() {
            fpath = dump_folder.join(
                now.format(&format!("%d_%m__%H_%M_%S({}).png", i))
                    .to_string(),
            );
            i += 1;
        }

        fpath
    }
}
