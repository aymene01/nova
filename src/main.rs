use env_logger::Builder;
use log::info;

fn main() {
    Builder::new().filter_level(log::LevelFilter::Info).init();
    info!("Hello, world nova!");
}
