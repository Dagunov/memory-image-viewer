use eframe::egui::*;
use mlua::prelude::*;
use serde::{Deserialize, Serialize};

use super::DataType;

const LUA_PRE: &'static str = "fout = io.tmpfile()";
const LUA_POST: &'static str = "fout:seek('set');lua_output = fout:read('a');fout:close()";

const LUA_PRE_FN: &'static str = "function process(r, g, b, a)\n";
const LUA_POST_FN: &'static str = "\nreturn r, g, b, a\nend";

fn lua_readify(input: &String) -> String {
    let res = input
        .replace("print()", r"fout:write('\n')")
        .replace("print", "fout:write");
    LUA_PRE_FN.to_owned() + &res + LUA_POST_FN
}

#[derive(Serialize, Deserialize)]
pub struct PostprocessingConfig {
    pub enabled: bool,
    code: String,
    #[serde(skip)]
    lua: Lua,
    last_msg: (String, Color32),
}

impl Default for PostprocessingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            code: String::new(),
            lua: Lua::new(),
            last_msg: (String::new(), Color32::BLACK),
        }
    }
}

impl PostprocessingConfig {
    /// returns true if should reprocess
    pub fn draw(&mut self, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.enabled, "Postprocessing");
            ui.add(Label::new("❔").selectable(false))
                .on_hover_ui(|ui| {
                    ui.label("Here you can write a postprocessing function in lua");
                    ui.label("Your function receives an input of r, g, b, a");
                    ui.label("Your function provides an output of r, g, b, a (same variables)");
                    ui.label("Both input and output are floats");
                    ui.label("Example code:\nl = r*0.33 + g*0.33 + b*0.33\nr, g, b = l, l, l");
                });
        });

        let mut successful_run = false;
        ui.add_enabled_ui(self.enabled, |ui| {
            if ui.code_editor(&mut self.code).changed() {
                successful_run = self.test_run_lua();
            }
            ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.colored_label(self.last_msg.1, &self.last_msg.0);
                });
        });
        return self.enabled && successful_run;
    }

    pub fn run_init(&mut self) {
        self.lua.load(LUA_PRE).exec().unwrap();

        if let Err(e) = self
            .lua
            .load(lua_readify(&self.code))
            .set_name("postprocessing")
            .exec()
        {
            self.last_msg = (format!("{}", e), Color32::RED);
        }
    }

    pub fn run_deinit(&mut self) {
        self.lua.load(LUA_POST).exec().unwrap();

        if let Ok(out) = self.lua.globals().get::<&str, String>("lua_output") {
            if out.is_empty() {
                self.last_msg.0 = String::from("All ok!");
                self.last_msg.1 = Color32::LIGHT_GREEN;
            } else {
                self.last_msg.0 = out;
                self.last_msg.1 = Color32::LIGHT_GREEN;
            }
        }
    }

    pub fn run(&self, bytes: &[u8], data_type: DataType) -> Result<Vec<u8>, ()> {
        let conversion_fn = match data_type {
            DataType::CV_32FC1 | DataType::CV_32FC2 | DataType::CV_32FC3 | DataType::CV_32FC4 => {
                |f: &[u8]| f32::from_ne_bytes(f.try_into().unwrap())
            }
            DataType::CV_64FC1 | DataType::CV_64FC2 | DataType::CV_64FC3 | DataType::CV_64FC4 => {
                |f: &[u8]| f64::from_ne_bytes(f.try_into().unwrap()) as f32
            }
            DataType::CV_16UC1 | DataType::CV_16UC2 | DataType::CV_16UC3 | DataType::CV_16UC4 => {
                |f: &[u8]| u16::from_ne_bytes(f.try_into().unwrap()) as f32
            }
            DataType::CV_8UC1 | DataType::CV_8UC2 | DataType::CV_8UC3 | DataType::CV_8UC4 => {
                |f: &[u8]| f[0] as f32
            }
        };
        let mut floats = [0f32; 4];
        let mut res = Vec::new();
        for pixel_bytes in bytes.chunks(data_type.bytes_per_pixel() as usize) {
            for (i, color_bytes) in pixel_bytes
                .chunks(data_type.bytes_per_color() as usize)
                .enumerate()
            {
                floats[i] = conversion_fn(color_bytes);
            }

            if let Ok(f) = self.lua.globals().get::<&str, LuaFunction>("process") {
                match f.call::<(f32, f32, f32, f32), (f32, f32, f32, f32)>((
                    floats[0], floats[1], floats[2], floats[3],
                )) {
                    Ok((r, g, b, a)) => {
                        floats[0] = r;
                        floats[1] = g;
                        floats[2] = b;
                        floats[3] = a;
                        for i in 0..data_type.channels() {
                            res.push((floats[i as usize] * u8::MAX as f32) as u8);
                        }
                    }
                    Err(e) => {
                        println!("{}", e);
                        return Err(());
                    }
                }
            } else {
                return Err(());
            }
        }
        return Ok(res);
    }

    /// returns if run was successful
    fn test_run_lua(&mut self) -> bool {
        self.lua.load(LUA_PRE).exec().unwrap();

        if let Err(e) = self
            .lua
            .load(lua_readify(&self.code))
            .set_name("postprocessing")
            .exec()
        {
            self.last_msg = (format!("{}", e), Color32::RED);
            return false;
        } else {
            self.lua.load(LUA_POST).exec().unwrap();

            if let Ok(out) = self.lua.globals().get::<&str, String>("lua_output") {
                if out.is_empty() {
                    self.last_msg.0 = String::from("All ok!");
                    self.last_msg.1 = Color32::LIGHT_GREEN;
                } else {
                    self.last_msg.0 = out;
                    self.last_msg.1 = Color32::LIGHT_GREEN;
                }
            }
            return true;
        }
    }
}
