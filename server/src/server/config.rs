//
// EPITECH PROJECT, 2026
// Zappy
// File description:
// Config
//

#[derive(Debug)]
pub struct Config {
    // Define Port Number
    pub port: u16,
    // Define Max Clients
    pub clients: usize,
    // Define Teams Names
    pub names: Vec<String>,
    // Define Time Unit
    pub frequency: u32,
    // Define World width
    pub x: usize,
    // Define World height
    pub y: usize,
}

impl Config {
    fn default() -> Self {
        Config {
            port: 4242,
            clients: 10,
            names: vec!["GRAPHICAL".to_string()],
            frequency: 1000,
            x: 10,
            y: 10,
        }
    }

    fn print_help() {
        println!("Zappy Server Configuration");
        println!();
        println!("USAGE:");
        println!("    zappy_server [OPTIONS]");
        println!();
        println!("OPTIONS:");
        println!("    -p <port>       Port number to listen on [default: 4242]");
        println!("    -c <clients>    Number of authorized clients per team [default: 10]");
        println!("    -n <names>...   Names of the teams (one or more) [default: GRAPHICAL]");
        println!("    -f <frequency>  Server frequency (time unit in ms) [default: 1000]");
        println!("    -x <x>          World width in tiles [default: 10]");
        println!("    -y <y>          World height in tiles [default: 10]");
        println!("    -h, --help      Print help information");
    }

    pub fn new() -> Self {
        let mut config = Config::default();
        let args: Vec<String> = std::env::args().skip(1).collect();
        let mut i = 0;

        while i < args.len() {
            match args[i].as_str() {
                "-h" => {
                    Config::print_help();
                    std::process::exit(0);
                }
                "-p" => {
                    i += 1;
                    config.port = args[i].parse().expect("invalid value for port");
                }
                "-c" => {
                    i += 1;
                    config.clients = args[i].parse().expect("invalid value for clients");
                }
                "-n" => {
                    let mut names = Vec::new();
                    while i + 1 < args.len() && !args[i + 1].starts_with('-') {
                        i += 1;
                        names.push(args[i].clone());
                    }
                    if !names.is_empty() {
                        config.names = names;
                    }
                }
                "-f" => {
                    i += 1;
                    config.frequency = args[i].parse().expect("invalid value for frequency");
                }
                "-x" => {
                    i += 1;
                    config.x = args[i].parse().expect("invalid value for x");
                }
                "-y" => {
                    i += 1;
                    config.y = args[i].parse().expect("invalid value for y");
                }
                other => panic!("unexpected argument: {other}"),
            }
            i += 1;
        }
        config
    }
}
