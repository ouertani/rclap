#![allow(dead_code)]
use clap::Parser;
use rclap::config;

#[config("config_with_inner.toml")]
struct MyConfig;

fn main() {
    let config = MyConfig::parse();
    println!("Config: {:#?}", config);
    println!("{}", &config.port);
    println!("{:?}", &config.url);
    println!("{:?}", &config.redis);
}
