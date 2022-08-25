pub mod logger;
pub mod ui;

use super::interpreter::Chip8;
use egui_macroquad;
use log::{debug, info, trace};
use macroquad::prelude::*;
use std::{cell::Cell, path::Path, rc::Rc, sync::mpsc::Receiver};

struct State {
    menu: ui::UserInterface,
    interpreter: Chip8,
    running: Rc<Cell<bool>>,
}

impl State {
    fn new(channel: Receiver<logger::Log>) -> Self {
        // This is bad but I want to get POC going
        let running = Rc::new(Cell::new(false));
        let ui_running = Rc::clone(&running);

        let mut chip8 = Chip8::new();
        chip8.load(Path::new("")).unwrap();
        Self {
            menu: ui::UserInterface::new(channel, ui_running),
            interpreter: chip8,
            running,
        }
    }
}

pub async fn run(logs: Receiver<logger::Log>) {
    let mut state = State::new(logs);
    loop {
        if state.running.get() {
            state.interpreter.cycle();
        }
        clear_background(WHITE);

        // Render egui
        egui_macroquad::ui(|egui_ctx| {
            egui::SidePanel::right("right panel")
                .min_width(screen_width() * 0.5)
                .resizable(false)
                .show(egui_ctx, |ui| {
                    state.menu.side_panel(ui);
                });
        });

        egui_macroquad::draw();

        // Draw Chip-8 screen
        let remaining_space = screen_width() * 0.33;
        let pixel_size = remaining_space / Chip8::VIDEO_WIDTH as f32;

        let buffer = state.interpreter.get_video_buffer();

        for x in 0..Chip8::VIDEO_WIDTH as usize {
            for y in 0..Chip8::VIDEO_HEIGHT as usize {
                if buffer[x + y * Chip8::VIDEO_HEIGHT as usize] == 0xFF {
                    trace!("Drawing square [{}, {}]", x, y);
                    draw_rectangle(
                        x as f32 * pixel_size,
                        y as f32 * pixel_size,
                        pixel_size,
                        pixel_size,
                        BLACK,
                    );
                }
            }
        }

        next_frame().await;
    }
}
