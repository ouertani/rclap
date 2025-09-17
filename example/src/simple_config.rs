#![allow(dead_code)]
use clap::Parser;
use rclap::config;

#[config]
struct MyConfig;

fn main() {
    let config = MyConfig::parse();
    println!("Config: {:#?}", config);
    println!("{}", &config.port);
    println!("{:?}", &config.ip);
}
