use clap::{Parser, ValueEnum};
use rclap::config;

#[config("second_config.toml")]
struct MyConfig;
#[config("config_external_clap.toml")]
struct SecondConfig;

fn main() {
    let config = MyConfig::parse();
    println!("Config2: {:#?}", config.redis);
    let config_ref = SecondConfig::parse();
    println!("Config: {:#?}", config_ref.redis);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum MyEnum {
    A,
    B,
    C,
}
