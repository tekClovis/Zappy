//
// EPITECH PROJECT, 2026
// Zappy
// File description:
// Config
//

const MIN_DIMENSION: usize = 1;
const MAX_TEAMS: usize = 100;
const MAX_CLIENTS_PER_TEAM: usize = 1000;

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub clients: usize,
    pub names: Vec<String>,
    pub frequency: u32,
    pub x: usize,
    pub y: usize,
}

impl Config {
    fn default() -> Self {
        Config {
            port: 4242,
            clients: 10,
            names: vec!["GRAPHICAL".to_string()],
            frequency: 100,
            x: 10,
            y: 10,
        }
    }

    fn print_help() {
        println!("Zappy Server Configuration\n");
        println!("USAGE:");
        println!("    zappy_server [OPTIONS]\n");
        println!("OPTIONS:");
        println!("    -p <port>       Port number to listen on [default: 4242]");
        println!(
            "    -c <clients>    Authorized clients per team, 1-{MAX_CLIENTS_PER_TEAM} [default: 10]"
        );
        println!(
            "    -n <names>...   Team names, up to {MAX_TEAMS} (one or more) [default: GRAPHICAL]"
        );
        println!("    -f <frequency>  Server frequency (time unit) [default: 100]");
        println!("    -x <x>          World width in tiles, min {MIN_DIMENSION} [default: 10]");
        println!("    -y <y>          World height in tiles, min {MIN_DIMENSION} [default: 10]");
        println!("    -h, --help      Print help information");
    }

    fn value<'a>(args: &'a [String], i: &mut usize, flag: &str) -> Result<&'a str, String> {
        *i += 1;
        args.get(*i)
            .map(String::as_str)
            .ok_or_else(|| format!("missing value for {flag}"))
    }

    pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<Self, String> {
        let mut config = Config::default();
        let args: Vec<String> = args.into_iter().collect();
        let mut i = 0;

        while i < args.len() {
            match args[i].as_str() {
                "-h" | "--help" => {
                    Config::print_help();
                    std::process::exit(0);
                }
                "-p" => {
                    config.port = Self::value(&args, &mut i, "-p")?
                        .parse()
                        .map_err(|_| "invalid value for -p (port)".to_string())?;
                }
                "-c" => {
                    config.clients = Self::value(&args, &mut i, "-c")?
                        .parse()
                        .map_err(|_| "invalid value for -c (clients)".to_string())?;
                }
                "-n" => {
                    let mut names = Vec::new();
                    while i + 1 < args.len() && !args[i + 1].starts_with('-') {
                        i += 1;
                        if args[i] == "GRAPHIC" {
                            return Err("team name cannot be GRAPHIC".to_string());
                        }
                        if names.contains(&args[i]) {
                            return Err(format!("duplicate team name: {}", args[i]));
                        }
                        names.push(args[i].clone());
                    }
                    if names.is_empty() {
                        return Err("at least one team name must be provided after -n".to_string());
                    }
                    config.names = names;
                }
                "-f" => {
                    config.frequency = Self::value(&args, &mut i, "-f")?
                        .parse()
                        .map_err(|_| "invalid value for -f (frequency)".to_string())?;
                }
                "-x" => {
                    config.x = Self::value(&args, &mut i, "-x")?
                        .parse()
                        .map_err(|_| "invalid value for -x (width)".to_string())?;
                }
                "-y" => {
                    config.y = Self::value(&args, &mut i, "-y")?
                        .parse()
                        .map_err(|_| "invalid value for -y (height)".to_string())?;
                }
                other => return Err(format!("unexpected argument: {other}")),
            }
            i += 1;
        }
        if config.x < MIN_DIMENSION || config.y < MIN_DIMENSION {
            return Err(format!(
                "world dimensions must be at least {MIN_DIMENSION}x{MIN_DIMENSION} (got {}x{})",
                config.x, config.y
            ));
        }
        if config.clients < 1 || config.clients > MAX_CLIENTS_PER_TEAM {
            return Err(format!(
                "clients per team must be between 1 and {MAX_CLIENTS_PER_TEAM} (got {})",
                config.clients
            ));
        }
        if config.names.len() > MAX_TEAMS {
            return Err(format!(
                "number of teams must be at most {MAX_TEAMS} (got {})",
                config.names.len()
            ));
        }
        Ok(config)
    }
}
