pub mod logger;
pub mod ui;

use log::info;
use mq::Shader;

use super::interpreter::Chip8;
use std::{cell::Cell, path::Path, rc::Rc, sync::mpsc::Receiver};
use {egui_miniquad as egui_mq, miniquad as mq};

#[repr(C)]
struct Vec2 {
    x: f32,
    y: f32,
}
#[repr(C)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}

struct Stage {
    pipeline: mq::Pipeline,
    bindings: mq::Bindings,
}

impl Stage {
    pub fn new(ctx: &mut mq::Context) -> Stage {
        #[rustfmt::skip]
        let vertices: [Vertex; 4] = [
            Vertex { pos : Vec2 { x: -0.5, y: -0.5 }, uv: Vec2 { x: 0., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y: -0.5 }, uv: Vec2 { x: 1., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y:  0.5 }, uv: Vec2 { x: 1., y: 1. } },
            Vertex { pos : Vec2 { x: -0.5, y:  0.5 }, uv: Vec2 { x: 0., y: 1. } },
        ];
        let vertex_buffer = mq::Buffer::immutable(ctx, mq::BufferType::VertexBuffer, &vertices);

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = mq::Buffer::immutable(ctx, mq::BufferType::IndexBuffer, &indices);

        let pixels: [u8; 4 * 4 * 4] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
            0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let texture = mq::Texture::from_rgba8(ctx, 4, 4, &pixels);

        let bindings = mq::Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![texture],
        };

        let shader = Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::meta()).unwrap();

        let pipeline = mq::Pipeline::new(
            ctx,
            &[mq::BufferLayout::default()],
            &[
                mq::VertexAttribute::new("pos", mq::VertexFormat::Float2),
                mq::VertexAttribute::new("uv", mq::VertexFormat::Float2),
            ],
            shader,
        );

        Stage { pipeline, bindings }
    }
}

struct State {
    egui_mq: egui_mq::EguiMq,
    menu: ui::UserInterface,
    interpreter: Chip8,
    running: Rc<Cell<bool>>,

    // Rendering stuff
    stage: Stage,
}

impl State {
    fn new(ctx: &mut mq::Context, channel: Receiver<logger::Log>) -> Self {
        // This is bad but I want to get POC going
        let running = Rc::new(Cell::new(false));
        let ui_running = Rc::clone(&running);

        let mut chip8 = Chip8::new();
        chip8.load(Path::new("")).unwrap();
        Self {
            egui_mq: egui_mq::EguiMq::new(ctx),
            menu: ui::UserInterface::new(channel, ui_running),
            interpreter: chip8,
            running,
            //
            stage: Stage::new(ctx),
        }
    }
}

impl mq::EventHandler for State {
    fn update(&mut self, _ctx: &mut mq::Context) {
        if self.running.get() {
            self.interpreter.cycle();
        }
    }

    fn draw(&mut self, ctx: &mut mq::Context) {
        // Clear screen, why are we doing this twice
        ctx.clear(Some((0., 1., 0., 1.)), None, None);
        ctx.begin_default_pass(mq::PassAction::clear_color(0.0, 0.0, 0.0, 0.1));
        ctx.end_render_pass();

        let screen_size = ctx.screen_size();

        // Generate EGUI
        self.egui_mq.run(ctx, |_ctx, egui_ctx| {
            egui::SidePanel::right("right panel")
                .min_width(screen_size.0 * 0.5)
                .resizable(false)
                .show(egui_ctx, |ui| {
                    self.menu.side_panel(ui);
                });
        });

        // Draw things behind EGUI here

        self.egui_mq.draw(ctx);

        // Draw things infront of EGUI here
        ctx.apply_pipeline(&self.stage.pipeline);
        ctx.apply_bindings(&self.stage.bindings);

        let remaining_space = screen_size.0 * 0.33;
        let display_size = (remaining_space, remaining_space / 2.0);
        let pixel_size = remaining_space / Chip8::VIDEO_WIDTH as f32;

        let buffer = self.interpreter.get_video_buffer();

        for x in 0..Chip8::VIDEO_WIDTH as usize {
            for y in 0..Chip8::VIDEO_HEIGHT as usize {
                if buffer[x + y * Chip8::VIDEO_HEIGHT as usize] == 0xFF {
                    info!("Drawing Pixel");
                    ctx.apply_uniforms(&shader::Uniforms {
                        offset: (x as f32 * pixel_size, y as f32 * pixel_size),
                    });
                    ctx.draw(0, 6, 1);
                }
            }
        }
        ctx.end_render_pass();
        ctx.commit_frame();
    }

    fn mouse_motion_event(&mut self, _ctx: &mut mq::Context, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(x, y);
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut mq::Context, dx: f32, dy: f32) {
        self.egui_mq.mouse_wheel_event(dx, dy);
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut mq::Context,
        button: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_down_event(ctx, button, x, y);
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut mq::Context,
        button: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_up_event(ctx, button, x, y);
    }

    fn char_event(&mut self, _ctx: &mut mq::Context, character: char, _: mq::KeyMods, _: bool) {
        // Forward event to EGUI
        self.egui_mq.char_event(character);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut mq::Context,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.key_down_event(ctx, keycode, keymods);
    }

    fn key_up_event(&mut self, _ctx: &mut mq::Context, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        self.egui_mq.key_up_event(keycode, keymods);
    }
}

pub fn run(logs: Receiver<logger::Log>) {
    miniquad::start(
        mq::conf::Conf {
            window_title: "Miniquad Test".to_string(),
            ..Default::default()
        },
        |ctx| Box::new(State::new(ctx, logs)),
    )
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 pos;
    attribute vec2 uv;
    uniform vec2 offset;
    varying lowp vec2 texcoord;
    void main() {
        gl_Position = vec4(pos + offset, 0, 1);
        texcoord = uv;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 texcoord;
    uniform sampler2D tex;
    void main() {
        gl_FragColor = texture2D(tex, texcoord);
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("offset", UniformType::Float2)],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub offset: (f32, f32),
    }
}
