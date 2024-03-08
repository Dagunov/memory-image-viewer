use std::path::PathBuf;

use eframe::egui::*;

use crate::imageprocessing::{ImageData, ImageProcessingResult};

use super::Toaster;

pub struct ImageView {
    image: ImageData,
    texture: TextureHandle,
    scale: f32,
    saving_thread: Option<std::thread::JoinHandle<ImageProcessingResult>>,
    toaster: Toaster,
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
        }
    }

    pub fn draw(&mut self, ui: &mut Ui) {
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
            if ui
                .add_enabled(self.saving_thread.is_none(), Button::new("Save"))
                .clicked()
            {
                match native_dialog::FileDialog::new()
                    .add_filter("png", &["png"])
                    .show_save_single_file()
                {
                    Ok(new_file_path) => {
                        if let Some(new_file_path) = new_file_path {
                            let image_copy = self.image.clone();
                            self.saving_thread =
                                Some(std::thread::spawn(move || image_copy.save(&new_file_path)));
                        } else {
                            self.toaster.info("Out file not selected", 3);
                        }
                    }
                    Err(native_dialog::Error::NoImplementation) => {
                        self.toaster.info(
                            "FileDialog Implementation not found, saving as out/out.png",
                            5,
                        );
                        let image_copy = self.image.clone();
                        self.saving_thread = Some(std::thread::spawn(move || {
                            image_copy.save(&PathBuf::from("out/out.png"))
                        }));
                    }
                    Err(e) => {
                        self.toaster.error(format!("FileDialog error: {:?}", e));
                    }
                }
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

        ui.add(Image::from_texture(&self.texture).max_size(vec2(
            self.image.width as f32 * self.scale,
            self.image.height as f32 * self.scale,
        )));
    }
}
