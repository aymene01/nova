use crate::config::session::Config;
use clap::Command;

pub fn parse_args() -> Option<Config> {
    let matches = Command::new("nova")
        .about("Robot Swarm Simulation")
        .version("0.1.0")
        .subcommand(Command::new("start").about("Start the simulation"))
        .get_matches();

    match matches.subcommand() {
        Some(("start", _)) => Some(Config::new()),
        _ => None,
    }
}
