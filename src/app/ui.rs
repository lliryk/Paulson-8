use std::{
    cell::Cell,
    rc::Rc,
    sync::{mpsc::Receiver, Arc},
};

use egui::{text::LayoutJob, Color32, ComboBox, Galley, ScrollArea, TextFormat, Ui};
use log::Level;
use macroquad::prelude::get_fps;

use super::logger;

pub struct UserInterface {
    channel: Receiver<logger::Log>,
    max_log_level: usize,
    logs: Vec<LogText>,
    running: Rc<Cell<bool>>,
}

impl UserInterface {
    pub fn new(channel: Receiver<logger::Log>, running: Rc<Cell<bool>>) -> Self {
        UserInterface {
            channel,
            max_log_level: 0,
            logs: Vec::new(),
            running,
        }
    }

    fn recieve_logs(&self) -> Vec<LogLayout> {
        let mut vec = Vec::new();
        for log in self.channel.try_iter() {
            let mut job = LayoutJob::default();

            let format = TextFormat {
                color: match log.level {
                    Level::Trace => Color32::GRAY,
                    Level::Debug => Color32::LIGHT_GRAY,
                    Level::Info => Color32::LIGHT_GREEN,
                    Level::Warn => Color32::YELLOW,
                    Level::Error => Color32::RED,
                },
                ..Default::default()
            };

            job.append("[", 0.0, TextFormat::default());
            job.append(&format!("{}", log.level), 0.0, format.clone());
            job.append("] - ", 0.0, TextFormat::default());

            job.append(&log.text, 0.0, format);

            vec.push(LogLayout {
                level: log.level,
                job,
            });
        }
        vec
    }

    pub fn side_panel(&mut self, ui: &mut Ui) {
        ui.heading(format!("FPS {}", get_fps()));
        let running = self.running.get();
        let btn_text = match running {
            true => "Stop",
            false => "Start",
        };
        if ui.button(btn_text).clicked() {
            self.running.set(!running);
        }
        if ui.button("Clear").clicked() {
            self.logs.clear();
        }

        ui.separator();
        let log_levels = [
            Level::Trace,
            Level::Debug,
            Level::Info,
            Level::Warn,
            Level::Error,
        ];
        ComboBox::from_label("Max Level").show_index(
            ui,
            &mut self.max_log_level,
            log_levels.len(),
            |i| log_levels[i].to_string(),
        );

        let unbaked_logs = self.recieve_logs();

        // Bake logs into properly layout text
        self.logs
            .extend(unbaked_logs.into_iter().map(|log_layout| LogText {
                level: log_layout.level,
                galley: ui.fonts().layout_job(log_layout.job),
            }));

        ScrollArea::vertical().show(ui, |ui| {
            for log in self
                .logs
                .iter()
                .filter(|log| log.level <= log_levels[self.max_log_level])
            {
                ui.painter().galley(
                    ui.next_widget_position() + egui::Vec2::new(20.0f32, 0.0f32),
                    Arc::clone(&log.galley),
                );
                // Create dummy to advance next widget position
                ui.label("");
            }
        });
    }
}

struct LogLayout {
    level: log::Level,
    job: LayoutJob,
}

struct LogText {
    level: log::Level,
    galley: Arc<Galley>,
}
