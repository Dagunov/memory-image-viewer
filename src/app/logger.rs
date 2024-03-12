use eframe::egui::{Color32, RichText, Ui};
use log::{error, info, warn};

pub struct Logger {
    msg: RichText,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            msg: RichText::new(""),
        }
    }
}

impl Logger {
    pub fn info(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        info!("{}", msg);
        self.msg = RichText::new(prepend_time(msg));
    }

    pub fn warn(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        warn!("{}", msg);
        self.msg = RichText::new(prepend_time(msg)).color(Color32::YELLOW);
    }

    pub fn error(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        error!("{}", msg);
        self.msg = RichText::new(prepend_time(msg)).color(Color32::RED);
    }

    pub fn show_label(&self, ui: &mut Ui) {
        ui.label(self.msg.clone());
    }
}

fn prepend_time(text: String) -> String {
    let now = chrono::Local::now();
    now.format("%H:%M.%S : ").to_string() + &text
}
