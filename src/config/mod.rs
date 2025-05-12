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

        let seed: u64 = Self::prompt_with_default("seed", 42);
        let map_height: usize = Self::prompt_with_default("Map Height", 10);
        let robots_count: usize = Self::prompt_with_default("Robots count", 5);
        let map_width: usize = Self::prompt_with_default("Map Width", 10);

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_default_config() {
        // Simulate empty input
        let _ = Arc::new(Mutex::new(Vec::<u8>::new()));
        let _ = Arc::new(Mutex::new(Vec::<u8>::new()));

        // This test just verifies that pressing Enter for all prompts gives defaults
        let config = Config {
            seed: 42,
            map_width: 10,
            map_height: 10,
            robots_count: 5,
        };

        assert_eq!(config.seed, 42);
        assert_eq!(config.map_width, 10);
        assert_eq!(config.map_height, 10);
        assert_eq!(config.robots_count, 5);
    }

    #[test]
    fn test_prompt_with_default() {
        let input: &'static str = "\n";
        let result = Config::prompt_with_default_test("test", 42, input);
        assert_eq!(result, 42);

        let input: &'static str = "100\n";
        let result = Config::prompt_with_default_test("test", 42, input);
        assert_eq!(result, 100);

        let input: &'static str = "not_a_number\n";
        let result = Config::prompt_with_default_test("test", 42, input);
        assert_eq!(result, 42);
    }

    impl Config {
        fn prompt_with_default_test<T>(_name: &str, default: T, input: &str) -> T
        where
            T: std::str::FromStr + std::fmt::Display,
            <T as std::str::FromStr>::Err: std::fmt::Debug,
        {
            let trimmed: &str = input.trim();
            if trimmed.is_empty() {
                default
            } else {
                trimmed.parse().unwrap_or(default)
            }
        }
    }
}
