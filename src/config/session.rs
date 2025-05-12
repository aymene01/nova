use std::io::{self, Write};

#[derive(Debug)]
pub struct Config {
    pub seed: u64,
    pub map_width: usize,
    pub map_height: usize,
    pub robots_count: usize,
}

impl Config {
    pub fn new() -> Config {
        println!(
            r#"
         _   _  ____ __      __
        | \ | |/ ___|\ \    / /
        |  \| | |     \ \  / /
        | |\  | |___   \ \/ /
        |_| \_|\____|   \__/

        ███╗   ██╗███╗   ██╗ █████╗
        ████╗  ██║████╗  ██║██╔══██╗
        ██╔██╗ ██║██╔██╗ ██║███████║
        ██║╚██╗██║██║╚██╗██║██╔══██║
        ██║ ╚████║██║ ╚████║██║  ██║
        ╚═╝  ╚═══╝╚═╝  ╚═══╝╚═╝  ╚═╝

        Welcome to Nova — your procedural robot simulation!
        "#
        );

        let seed = Self::prompt_with_default("seed", 42);
        let map_height = Self::prompt_with_default("Map Height", 10);
        let robots_count = Self::prompt_with_default("Robots count", 5);
        let map_width = Self::prompt_with_default("Map Width", 10);

        Config {
            map_height,
            map_width,
            robots_count,
            seed,
        }
    }

    pub fn prompt_with_default<T>(name: &str, default: T) -> T
    where
        T: std::str::FromStr + std::fmt::Display,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
    {
        print!("{} [{}]: ", name, default);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return default;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            default
        } else {
            trimmed.parse().unwrap_or_else(|_| {
                println!("Invalid input, using default");
                default
            })
        }
    }
}
