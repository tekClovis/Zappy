//
// EPITECH PROJECT, 2026
// Zappy
// File description:
// Config
//
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about = "Zappy Server Configuration")]
pub struct Config {
    // Define Port Number
    #[arg(short, long, default_value = "4242", help = "Port number to listen on")]
    pub port: u16,
    // Define Max Clients 
    #[arg(short, long, default_value = "10", help = "Number of authorized clients per team")]
    pub clients: usize,
    // Define Teams Names
    #[arg(required = true, short, long, num_args = 1.., default_value = "GRAPHICAL", help = "Names of the teams (one or more)")]
    pub names: Vec<String>,
    // Define Time Unit 
    #[arg(short, long, default_value = "1000", help = "Server frequency (time unit in ms)")]
    pub frequency: u32,
    // Define World width 
    #[arg(short, long, default_value = "10", help = "World width in tiles")]
    pub x: usize,
    // Define World height 
    #[arg(short, long, default_value = "10", help = "World height in tiles")]
    pub y: usize,
}

impl Config {
    pub fn new() -> Self {
        Config::parse()
    }
}