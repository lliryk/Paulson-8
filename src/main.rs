use paulson_8::app::{self, logger};

use macroquad::prelude::*;

#[allow(dead_code)]
fn window_conf() -> Conf {
    Conf {
        window_title: "Paulson-8".to_owned(),
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Make this based on env variable
    let log = logger::init(log::LevelFilter::Trace).expect("Logger faild to initalize");
    app::run(log).await;
}
