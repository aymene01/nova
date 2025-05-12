use log::{info};
use env_logger::Builder;

fn main() {
    Builder::new().filter_level(log::LevelFilter::Info).init();
    info!("Hello, world nova!");
}
