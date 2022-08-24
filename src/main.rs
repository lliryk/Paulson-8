use paulson_8::app::{self, logger};

fn main() {
    // Make this based on env variable
    let log = logger::init(log::LevelFilter::Trace).expect("Logger faild to initalize");
    app::run(log);
}
